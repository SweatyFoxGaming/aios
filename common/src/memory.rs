//! Semantic memory types for Phoenix OS (Mnemosyne).

use alloc::string::String;

/// Represents a concept or entity in the knowledge graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeNode {
    /// Unique identifier for the node.
    pub id: u64,
    /// The name/label of the concept.
    pub label: String,
}

/// Represents a relationship between two knowledge nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// A is a subset or instance of B.
    IsA,
    /// A is part of B.
    PartOf,
    /// A is owned by B.
    OwnedBy,
    /// A is related to B in a general way.
    RelatedTo,
}

/// A connection in the knowledge graph.
#[derive(Debug, Clone)]
pub struct Edge {
    /// Source node ID.
    pub from: u64,
    /// Target node ID.
    pub to: u64,
    /// The type of relationship.
    pub relation: Relation,
}
