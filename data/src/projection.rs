//! The Markdown projection seam.
//!
//! Tier 2 holds a document as Loro containers; Tier 1 owns the Markdown
//! dialect. This trait is the only door between them, so neither tier imports
//! the other's internals.

use crate::error::Result;
use crate::types::{Block, BlockId, DocPath, PropertyValue};

/// The in-memory shape of a document, shared by Tier 1 and Tier 2.
///
/// `Tier 1 → Tier 2`: parsing a `.md` file yields this; Tier 2 turns it into
/// Loro containers. `Tier 2 → Tier 1`: projecting a Loro document yields this;
/// Tier 1 renders it back to Markdown.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DocumentSnapshot {
    /// Frontmatter key-values, in authored order.
    pub frontmatter: Vec<(String, PropertyValue)>,
    /// Body blocks, in document order.
    pub blocks: Vec<Block>,
}

impl DocumentSnapshot {
    /// Look a frontmatter property up.
    pub fn property(&self, key: &str) -> Option<&PropertyValue> {
        self.frontmatter
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// Look a frontmatter property up as a string, ignoring other value types.
    pub fn string_property(&self, key: &str) -> Option<String> {
        match self.property(key)? {
            PropertyValue::String(value) => Some(value.clone()),
            _ => None,
        }
    }

    /// The block with the given id.
    pub fn block(&self, id: &BlockId) -> Option<&Block> {
        self.blocks.iter().find(|block| &block.id == id)
    }
}

/// Markdown dialect projection: parse a file, render a document.
pub trait MarkdownProjection: Send + Sync {
    /// Parse a `.md` file's source into frontmatter and blocks.
    fn parse(&self, doc: &DocPath, source: &str) -> Result<DocumentSnapshot>;

    /// Render a document back to CommonMark + GFM with YAML frontmatter.
    ///
    /// `linked_blocks` are the block ids that must be written as
    /// `<!-- ^id -->` anchors because something links to them.
    fn render(
        &self,
        doc: &DocPath,
        snapshot: &DocumentSnapshot,
        linked_blocks: &[String],
    ) -> Result<String>;
}
