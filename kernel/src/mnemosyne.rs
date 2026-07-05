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
}

lazy_static! {
    // Preallocated so `nodes.push`/`edges.push` below never need to grow
    // their `Vec` -- growing an existing heap allocation crashes on this
    // target (see safe_alloc.rs).
    static ref GRAPH: Mutex<KnowledgeGraph> = Mutex::new(KnowledgeGraph {
        nodes: Vec::with_capacity(64),
        edges: Vec::with_capacity(64),
    });
}

/// Add a concept node to the knowledge graph.
#[must_use]
pub fn add_node(label: String) -> u64 {
    let mut graph = GRAPH.lock();
    let id = graph.nodes.len() as u64;
    graph.nodes.push(KnowledgeNode { id, label });
    id
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
