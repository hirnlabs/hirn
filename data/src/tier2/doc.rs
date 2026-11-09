//! The Loro document schema: `frontmatter` map plus `blocks` movable list.

use loro::{
    ExportMode, LoroDoc, LoroMap, LoroMovableList, LoroResult, LoroText, LoroValue,
    ValueOrContainer,
};

use crate::error::{Error, Result};
use crate::projection::DocumentSnapshot;
use crate::types::{Block, BlockId, BlockKind, PropertyValue};

/// Root container holding frontmatter key-values.
pub const FRONTMATTER: &str = "frontmatter";
/// Root container holding the block list.
pub const BLOCKS: &str = "blocks";

/// A document's Loro state.
pub struct LoroDocument {
    doc: LoroDoc,
}

impl LoroDocument {
    /// A new, empty document.
    pub fn new() -> Self {
        Self {
            doc: LoroDoc::new(),
        }
    }

    /// Rebuild a document from a snapshot, replacing whatever it held.
    pub fn from_snapshot(snapshot: &DocumentSnapshot) -> Result<Self> {
        let document = Self::new();
        document.load_snapshot(snapshot)?;
        Ok(document)
    }

    /// Import a persisted snapshot or update blob.
    pub fn import(bytes: &[u8]) -> Result<Self> {
        let doc = LoroDoc::new();
        doc.import(bytes)?;
        Ok(Self { doc })
    }

    /// The underlying Loro document.
    pub fn raw(&self) -> &LoroDoc {
        &self.doc
    }

    /// The frontmatter map.
    pub fn frontmatter(&self) -> LoroMap {
        self.doc.get_map(FRONTMATTER)
    }

    /// The block list.
    pub fn blocks(&self) -> LoroMovableList {
        self.doc.get_movable_list(BLOCKS)
    }

    /// Read the current state as a plain snapshot.
    pub fn snapshot(&self) -> Result<DocumentSnapshot> {
        snapshot_from_doc(&self.doc)
    }

    /// Replace the whole content with `snapshot`.
    pub fn load_snapshot(&self, snapshot: &DocumentSnapshot) -> Result<()> {
        let frontmatter = self.frontmatter();
        frontmatter.clear()?;
        for (key, value) in &snapshot.frontmatter {
            frontmatter.insert(key, loro_value(value))?;
        }

        let blocks = self.blocks();
        blocks.clear()?;
        for block in &snapshot.blocks {
            push_block(&blocks, block)?;
        }
        self.doc.commit();
        Ok(())
    }

    /// Append a block and return its id.
    pub fn append_block(&self, kind: BlockKind, text: &str) -> Result<BlockId> {
        let block = Block::new(BlockId::generate(), kind, text);
        let blocks = self.blocks();
        push_block(&blocks, &block)?;
        self.doc.commit();
        Ok(block.id)
    }

    /// Insert a block with an existing id and metadata at `index`.
    pub fn insert_block_with_id_at(
        &self,
        index: usize,
        id: BlockId,
        kind: BlockKind,
        text: &str,
        meta: serde_json::Map<String, serde_json::Value>,
    ) -> Result<BlockId> {
        let block = Block {
            id,
            kind,
            text: text.to_string(),
            meta,
        };
        let blocks = self.blocks();
        let map = block_map(&block)?;
        let insert_pos = index.min(blocks.len());
        blocks.insert_container(insert_pos, map)?;
        self.doc.commit();
        Ok(block.id)
    }

    /// Insert a block at `index`, minted with a fresh id.
    pub fn insert_block_at(&self, index: usize, kind: BlockKind, text: &str) -> Result<BlockId> {
        self.insert_block_with_id_at(
            index,
            BlockId::generate(),
            kind,
            text,
            serde_json::Map::new(),
        )
    }

    /// Delete the block at `index`.
    pub fn delete_block_at(&self, index: usize) -> Result<()> {
        self.blocks().delete(index, 1)?;
        self.doc.commit();
        Ok(())
    }

    /// Replace the text of the block at `index`.
    pub fn set_block_text_at(&self, index: usize, text: &str) -> Result<()> {
        let blocks = self.blocks();
        let text_container = text_container_at(&blocks, index)?;
        replace_text(&text_container, text)?;
        self.doc.commit();
        Ok(())
    }

    /// Move a block from one position to another.
    pub fn move_block(&self, from: usize, to: usize) -> Result<()> {
        self.blocks().mov(from, to)?;
        self.doc.commit();
        Ok(())
    }

    /// Delete a block by id.
    pub fn delete_block(&self, id: &BlockId) -> Result<()> {
        let blocks = self.blocks();
        let Some(index) = index_of(&blocks, id)? else {
            return Err(Error::NotFound(format!("block {id}")));
        };
        blocks.delete(index, 1)?;
        self.doc.commit();
        Ok(())
    }

    /// Set a frontmatter property.
    pub fn set_property(&self, key: &str, value: &PropertyValue) -> Result<()> {
        self.frontmatter().insert(key, loro_value(value))?;
        self.doc.commit();
        Ok(())
    }

    /// Remove a frontmatter property.
    pub fn delete_property(&self, key: &str) -> Result<()> {
        self.frontmatter().delete(key)?;
        self.doc.commit();
        Ok(())
    }

    /// Set a block's text, replacing whatever it held.
    pub fn set_block_text(&self, id: &BlockId, text: &str) -> Result<()> {
        let blocks = self.blocks();
        let index = index_of(&blocks, id)?.ok_or_else(|| Error::NotFound(format!("block {id}")))?;
        let text_container = text_container_at(&blocks, index)?;
        replace_text(&text_container, text)?;
        self.doc.commit();
        Ok(())
    }

    /// Export the full state as a compact snapshot, preserving the version
    /// vector so peers can still compute deltas against it.
    pub fn export_snapshot(&self) -> Result<Vec<u8>> {
        self.doc.commit();
        Ok(self.doc.export(ExportMode::Snapshot)?)
    }

    /// Export only the operations a peer holding `their` version is missing.
    pub fn export_updates(&self, their: &loro::VersionVector) -> Result<Vec<u8>> {
        Ok(self.doc.export(ExportMode::updates(their))?)
    }

    /// Apply remote operations.
    pub fn apply_updates(&self, bytes: &[u8]) -> Result<()> {
        self.doc.import(bytes)?;
        Ok(())
    }

    /// Number of operations in the op log, the compaction trigger.
    pub fn oplog_len(&self) -> usize {
        self.doc.len_ops()
    }

    /// The complete version vector, kept across compaction.
    pub fn version_vector(&self) -> loro::VersionVector {
        self.doc.oplog_vv()
    }

    /// Force a commit so pending local ops become exportable.
    pub fn commit(&self) {
        self.doc.commit();
    }
}

impl Default for LoroDocument {
    fn default() -> Self {
        Self::new()
    }
}

fn loro_value(value: &PropertyValue) -> LoroValue {
    match value {
        PropertyValue::Null => LoroValue::Null,
        PropertyValue::Bool(v) => LoroValue::Bool(*v),
        PropertyValue::Number(v) if v.fract() == 0.0 && v.is_finite() => LoroValue::I64(*v as i64),
        PropertyValue::Number(v) => LoroValue::Double(*v),
        PropertyValue::String(v) => LoroValue::String(v.as_str().into()),
        PropertyValue::List(items) => LoroValue::List(items.iter().map(loro_value).collect()),
        PropertyValue::Map(map) => LoroValue::Map(
            map.iter()
                .map(|(k, v)| (k.as_str().into(), loro_value(&PropertyValue::from_json(v))))
                .collect(),
        ),
    }
}

/// Build the Loro map for one block.
///
/// An empty meta map has no state to persist; skipping it keeps the container
/// tree free of maps that would vanish on export.
fn block_map(block: &Block) -> LoroResult<LoroMap> {
    let map = LoroMap::new();
    map.insert("id", block.id.as_str())?;
    map.insert("kind", block.kind.as_str())?;
    let text = LoroText::new();
    text.insert(0, &block.text)?;
    map.insert_container(Block::TEXT_KEY, text)?;
    if !block.meta.is_empty() {
        let meta = LoroMap::new();
        for (key, value) in &block.meta {
            meta.insert(key, loro_value(&PropertyValue::from_json(value)))?;
        }
        map.insert_container("meta", meta)?;
    }
    Ok(map)
}

fn push_block(blocks: &LoroMovableList, block: &Block) -> LoroResult<()> {
    blocks.push_container(block_map(block)?)?;
    Ok(())
}

/// The `text` Loro container of the block at `index`.
fn text_container_at(blocks: &LoroMovableList, index: usize) -> Result<LoroText> {
    let Some(ValueOrContainer::Container(container)) = blocks.get(index) else {
        return Err(Error::NotFound(format!("block at {index}")));
    };
    let map = container
        .into_map()
        .map_err(|_| Error::Loro(format!("block at {index} is not a map")))?;
    map.get(Block::TEXT_KEY)
        .and_then(|value| match value {
            ValueOrContainer::Container(container) => container.into_text().ok(),
            ValueOrContainer::Value(_) => None,
        })
        .ok_or_else(|| Error::Loro(format!("block at {index} has no text container")))
}

fn replace_text(text: &LoroText, replacement: &str) -> Result<()> {
    let current = text.to_string();
    if current == replacement {
        return Ok(());
    }
    let len = current.chars().count();
    if len > 0 {
        text.delete(0, len)?;
    }
    if !replacement.is_empty() {
        text.insert(0, replacement)?;
    }
    Ok(())
}

fn index_of(blocks: &LoroMovableList, id: &BlockId) -> Result<Option<usize>> {
    for index in 0..blocks.len() {
        if let Some(ValueOrContainer::Container(container)) = blocks.get(index) {
            let Ok(map) = container.into_map() else {
                continue;
            };
            if let Some(ValueOrContainer::Value(LoroValue::String(found))) = map.get("id")
                && found.as_str() == id.as_str()
            {
                return Ok(Some(index));
            }
        }
    }
    Ok(None)
}

/// Convert a Loro document into a plain snapshot.
pub fn snapshot_from_doc(doc: &LoroDoc) -> Result<DocumentSnapshot> {
    Ok(snapshot_from_json(doc.get_deep_value()))
}

/// Convert a Loro document's deep JSON value into a snapshot.
///
/// Reads the whole document in one pass and converts the numbers through
/// `serde_json`, so an integral `I64` stays an integer instead of coming back
/// as `1.0`.
pub fn snapshot_from_json(value: LoroValue) -> DocumentSnapshot {
    let json = serde_json::to_value(&value).unwrap_or(serde_json::Value::Null);
    let frontmatter = json
        .get(FRONTMATTER)
        .and_then(serde_json::Value::as_object)
        .map(|map| {
            map.iter()
                .map(|(key, value)| (key.clone(), PropertyValue::from_json(value)))
                .collect()
        })
        .unwrap_or_default();

    let blocks = json
        .get(BLOCKS)
        .and_then(serde_json::Value::as_array)
        .map(|items| items.iter().filter_map(block_from_json).collect())
        .unwrap_or_default();

    DocumentSnapshot {
        frontmatter,
        blocks,
    }
}

fn block_from_json(value: &serde_json::Value) -> Option<Block> {
    let object = value.as_object()?;
    let id = object
        .get("id")
        .and_then(serde_json::Value::as_str)
        .map_or_else(BlockId::generate, BlockId::new);
    let kind = object
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .map_or(BlockKind::Paragraph, BlockKind::from_raw);
    let text = object
        .get(Block::TEXT_KEY)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let meta = object
        .get("meta")
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_default();
    Some(Block {
        id,
        kind,
        text,
        meta,
    })
}

/// Build a document from a snapshot.
pub fn doc_from_snapshot(snapshot: &DocumentSnapshot) -> Result<LoroDocument> {
    LoroDocument::from_snapshot(snapshot)
}
