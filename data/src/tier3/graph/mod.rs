//! Grafeo graph tier: `<project>/.hirn/graph.grafeo`.
//!
//! Nodes are `:Document { path, title, type, is_ghost }` and `:Block { id,
//! kind }`. Edges are `:LINKS_TO` from `[[wikilinks]]` plus typed edges from
//! frontmatter references. Missing link targets become ghost documents so
//! incoming backlinks survive (HIR-73 resolution).

use std::collections::HashMap;
use std::path::Path;

use grafeo::GrafeoDB;
use grafeo::Value;

use crate::error::{Error, Result};
use crate::tier1::links::normalize_doc_path;
use crate::types::{Block, DocPath, TypeName};

pub use crate::tier1::links::{ExtractedLink, extract_links};

/// The graph index for one project.
pub struct GraphIndex {
    db: GrafeoDB,
}

impl GraphIndex {
    /// Open or create `<project>/.hirn/graph.grafeo`.
    pub fn open(project_root: impl AsRef<Path>) -> Result<Self> {
        let root = project_root.as_ref();
        let hirn_dir = root.join(".hirn");
        if !hirn_dir.exists() {
            std::fs::create_dir_all(&hirn_dir).map_err(|e| Error::io(&hirn_dir, e))?;
        }
        Ok(Self {
            db: GrafeoDB::open(hirn_dir.join("graph.grafeo"))?,
        })
    }

    /// An in-memory graph, for tests.
    pub fn in_memory() -> Self {
        Self {
            db: GrafeoDB::new_in_memory(),
        }
    }

    /// Index `Document.path` and `Block.doc_path` so per-document lookups do
    /// not scan the graph.
    ///
    /// Safe to call on every open: Grafeo keeps the index once created.
    pub fn ensure_indexes(&self) -> Result<()> {
        if !self.db.has_property_index("path") {
            self.db.create_property_index("path")?;
        }
        if !self.db.has_property_index("doc_path") {
            self.db.create_property_index("doc_path")?;
        }
        Ok(())
    }

    /// The underlying engine handle.
    pub fn engine(&self) -> &GrafeoDB {
        &self.db
    }

    /// Replace one document's node properties, owned block nodes and outgoing
    /// edges.
    ///
    /// Inbound edges are deliberately preserved: another document's backlink
    /// to this one must survive a re-index. The source node is updated in
    /// place rather than deleted.
    pub fn reindex_document(
        &mut self,
        doc: &DocPath,
        type_name: Option<&TypeName>,
        title: Option<&str>,
        blocks: &[Block],
        links: &[ExtractedLink],
    ) -> Result<()> {
        let source = self.upsert_document_node(doc, type_name, title, false)?;
        self.delete_outgoing_edges(source)?;
        self.delete_owned_blocks(doc)?;

        for block in blocks {
            let block_node = self.db.create_node_with_props(
                &["Block"],
                [
                    ("id", Value::from(block.id.as_str())),
                    ("kind", Value::from(block.kind.as_str())),
                    ("doc_path", Value::from(doc.as_str())),
                ],
            )?;
            self.db.create_edge(source, block_node, "CONTAINS")?;
        }
        for link in links {
            let target_doc = DocPath::from(normalize_doc_path(&link.target));
            let target = self.upsert_document_node(&target_doc, None, None, true)?;
            self.db.create_edge_with_props(
                source,
                target,
                &link.relation,
                [
                    (
                        "anchor",
                        Value::from(link.anchor.clone().unwrap_or_default()),
                    ),
                    ("alias", Value::from(link.alias.clone().unwrap_or_default())),
                ],
            )?;
        }
        self.clean_orphan_ghosts()
    }

    /// Delete one document's node, its owned `:Block` nodes, and every edge
    /// touching them. Ghost targets that no longer have a backlink go too.
    pub fn delete_document(&mut self, doc: &DocPath) -> Result<()> {
        self.delete_owned_blocks(doc)?;
        if let Some(id) = self.find_document(doc)? {
            self.db.detach_delete_node(id)?;
        }
        self.clean_orphan_ghosts()
    }

    /// Remove every node and edge, for a full rebuild.
    pub fn clear(&mut self) -> Result<()> {
        let result = self
            .db
            .execute_language("MATCH (n) RETURN id(n) AS id", "cypher", None)?;
        let ids: Vec<grafeo::NodeId> = result
            .rows()
            .iter()
            .filter_map(|row| match row.first() {
                Some(Value::Int64(id)) => u64::try_from(*id).ok().map(grafeo::NodeId::new),
                _ => None,
            })
            .collect();
        for id in ids {
            self.db.detach_delete_node(id)?;
        }
        Ok(())
    }

    /// Run an openCypher query and return the rows as JSON.
    pub fn query(&self, cypher: &str) -> Result<Vec<serde_json::Value>> {
        let result = self.db.execute_language(cypher, "cypher", None)?;
        let columns = result.columns.clone();
        let mut out = Vec::new();
        for row in result.rows() {
            let mut object = serde_json::Map::new();
            for (idx, name) in columns.iter().enumerate() {
                object.insert(name.clone(), value_to_json(row.get(idx)));
            }
            out.push(serde_json::Value::Object(object));
        }
        Ok(out)
    }

    fn find_document(&self, doc: &DocPath) -> Result<Option<grafeo::NodeId>> {
        let mut params = HashMap::new();
        params.insert("path".to_string(), Value::from(doc.as_str()));
        let result = self.db.execute_with_params(
            "MATCH (d:Document {path: $path}) RETURN id(d) AS id LIMIT 1",
            params,
        )?;
        let Some(row) = result.rows().first() else {
            return Ok(None);
        };
        Ok(match row.first() {
            Some(Value::Int64(id)) => Some(grafeo::NodeId::new(u64::try_from(*id).unwrap_or(0))),
            _ => None,
        })
    }

    fn delete_outgoing_edges(&mut self, node: grafeo::NodeId) -> Result<()> {
        let mut params = HashMap::new();
        params.insert("id".to_string(), Value::from(node.0 as i64));
        let result = self.db.execute_with_params(
            "MATCH (d) WHERE id(d) = $id MATCH (d)-[r]->() RETURN id(r) AS id",
            params,
        )?;
        let ids: Vec<grafeo::EdgeId> = result
            .rows()
            .iter()
            .filter_map(|row| match row.first() {
                Some(Value::Int64(id)) => u64::try_from(*id).ok().map(grafeo::EdgeId::new),
                _ => None,
            })
            .collect();
        for id in ids {
            self.db.delete_edge(id)?;
        }
        Ok(())
    }

    fn delete_owned_blocks(&mut self, doc: &DocPath) -> Result<()> {
        let mut params = HashMap::new();
        params.insert("path".to_string(), Value::from(doc.as_str()));
        let result = self.db.execute_with_params(
            "MATCH (b:Block {doc_path: $path}) RETURN id(b) AS id",
            params,
        )?;
        let ids: Vec<grafeo::NodeId> = result
            .rows()
            .iter()
            .filter_map(|row| match row.first() {
                Some(Value::Int64(id)) => u64::try_from(*id).ok().map(grafeo::NodeId::new),
                _ => None,
            })
            .collect();
        for id in ids {
            self.db.detach_delete_node(id)?;
        }
        Ok(())
    }

    /// Delete ghost documents that no longer have any edge.
    fn clean_orphan_ghosts(&mut self) -> Result<()> {
        let result = self.db.execute_language(
            "MATCH (d:Document {is_ghost: true}) RETURN d.path AS path",
            "cypher",
            None,
        )?;
        let paths: Vec<String> = result
            .rows()
            .iter()
            .filter_map(|row| match row.first() {
                Some(Value::String(path)) => Some(path.as_str().to_string()),
                _ => None,
            })
            .collect();
        for path in paths {
            let mut params = HashMap::new();
            params.insert("path".to_string(), Value::from(path.as_str()));
            let degree = self.db.execute_with_params(
                "MATCH (d:Document {path: $path})-[r]-() RETURN count(r) AS n",
                params,
            )?;
            let connected = degree
                .rows()
                .first()
                .and_then(|row| row.first())
                .and_then(|value| match value {
                    Value::Int64(n) => Some(*n),
                    _ => None,
                })
                .unwrap_or(0);
            if connected == 0
                && let Some(id) = self.find_document(&DocPath::new(path))?
            {
                self.db.detach_delete_node(id)?;
            }
        }
        Ok(())
    }

    fn upsert_document_node(
        &mut self,
        doc: &DocPath,
        type_name: Option<&TypeName>,
        title: Option<&str>,
        ghost: bool,
    ) -> Result<grafeo::NodeId> {
        if let Some(existing) = self.find_document(doc)? {
            if !ghost {
                self.db
                    .set_node_property(existing, "is_ghost", Value::from(false))?;
                self.db.set_node_property(
                    existing,
                    "type",
                    Value::from(type_name.map(TypeName::as_str).unwrap_or_default()),
                )?;
                self.db.set_node_property(
                    existing,
                    "title",
                    Value::from(title.unwrap_or_else(|| doc.stem())),
                )?;
            }
            return Ok(existing);
        }
        let id = self.db.create_node_with_props(
            &["Document"],
            [
                ("path", Value::from(doc.as_str())),
                ("title", Value::from(title.unwrap_or_else(|| doc.stem()))),
                (
                    "type",
                    Value::from(type_name.map(TypeName::as_str).unwrap_or_default()),
                ),
                ("is_ghost", Value::from(ghost)),
            ],
        )?;
        Ok(id)
    }
}

fn value_to_json(value: Option<&Value>) -> serde_json::Value {
    match value {
        None | Some(Value::Null) => serde_json::Value::Null,
        Some(Value::Bool(v)) => serde_json::Value::from(*v),
        Some(Value::Int64(v)) => serde_json::Value::from(*v),
        Some(Value::Float64(v)) => serde_json::Value::from(*v),
        Some(Value::String(v)) => serde_json::Value::from(v.as_str().to_string()),
        Some(other) => serde_json::Value::from(format!("{other:?}")),
    }
}
