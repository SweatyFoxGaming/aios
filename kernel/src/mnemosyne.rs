//! Mnemosyne: Semantic memory and knowledge graph for Phoenix OS.

use crate::println;
use alloc::string::String;
use alloc::vec::Vec;
use common::memory::{Edge, KnowledgeNode, Relation};
use lazy_static::lazy_static;
use spin::Mutex;

/// The knowledge graph store.
struct KnowledgeGraph {
    nodes: Vec<KnowledgeNode>,
    edges: Vec<Edge>,
    next_id: u64,
}

lazy_static! {
    static ref GRAPH: Mutex<KnowledgeGraph> = Mutex::new(KnowledgeGraph {
        nodes: Vec::new(),
        edges: Vec::new(),
        next_id: 0,
    });
}

/// Add a concept node to the knowledge graph.
#[must_use]
pub fn add_node(label: String) -> u64 {
    let mut graph = GRAPH.lock();
    let id = graph.next_id;
    graph.next_id += 1;
    graph.nodes.push(KnowledgeNode {
        id,
        label,
        significance: 1.0, // Default to full significance
        last_access: 0,    // Initial timestamp
    });
    id
}

/// Retrieve a node and update its significance and access time.
pub fn touch_node(id: u64) {
    let mut graph = GRAPH.lock();
    if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == id) {
        node.last_access += 1; // Increment simulated timestamp
        node.significance = (node.significance + 0.1).min(1.0);
    }
}

/// Remove nodes that fall below a certain significance threshold.
pub fn prune_nodes(threshold: f32) -> usize {
    let mut graph = GRAPH.lock();
    let initial_count = graph.nodes.len();

    // Keep nodes above threshold
    graph.nodes.retain(|n| n.significance >= threshold);

    let removed_count = initial_count - graph.nodes.len();

    // Clean up edges that reference removed nodes
    if removed_count > 0 {
        let valid_ids: Vec<u64> = graph.nodes.iter().map(|n| n.id).collect();
        graph
            .edges
            .retain(|e| valid_ids.contains(&e.from) && valid_ids.contains(&e.to));
    }

    removed_count
}

/// Decay significance of all nodes.
pub fn decay_significance(amount: f32) {
    let mut graph = GRAPH.lock();
    for node in &mut graph.nodes {
        node.significance = (node.significance - amount).max(0.0);
    }
}

/// Add a relationship between two nodes.
pub fn add_relation(from: u64, to: u64, relation: Relation) {
    let mut graph = GRAPH.lock();
    graph.edges.push(Edge { from, to, relation });
}

/// Print the current knowledge graph.
pub fn debug_graph() {
    let graph = GRAPH.lock();
    println!("--- Mnemosyne Knowledge Graph ---");
    for edge in &graph.edges {
        #[allow(clippy::cast_possible_truncation)]
        let from_label = graph
            .nodes
            .get(edge.from as usize)
            .map_or("?", |n| &n.label);
        #[allow(clippy::cast_possible_truncation)]
        let to_label = graph.nodes.get(edge.to as usize).map_or("?", |n| &n.label);
        println!("{from_label} --({:?})--> {to_label}", edge.relation);
    }
    println!("---------------------------------");
}
