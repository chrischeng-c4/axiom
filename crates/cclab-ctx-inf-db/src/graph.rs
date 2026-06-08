use std::collections::{HashMap, HashSet, VecDeque};

use crate::engine::CtxInfEngine;
use crate::error::Result;
use crate::types::*;

/// Graph traversal and analytics operations on the engine.
impl CtxInfEngine {
    /// BFS: find all entities reachable from `start` within `max_hops`.
    pub fn reachable(
        &self,
        start: EntityId,
        max_hops: usize,
        direction: Direction,
    ) -> Result<Vec<(EntityId, usize)>> {
        let mut visited: HashMap<EntityId, usize> = HashMap::new();
        let mut queue: VecDeque<(EntityId, usize)> = VecDeque::new();

        visited.insert(start, 0);
        queue.push_back((start, 0));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_hops {
                continue;
            }

            let neighbor_ids = self.raw_neighbor_ids(current, direction);
            for neighbor in neighbor_ids {
                if !visited.contains_key(&neighbor) {
                    let next_depth = depth + 1;
                    visited.insert(neighbor, next_depth);
                    queue.push_back((neighbor, next_depth));
                }
            }
        }

        // Remove start from result.
        visited.remove(&start);
        let mut result: Vec<_> = visited.into_iter().collect();
        result.sort_by_key(|(_, d)| *d);
        Ok(result)
    }

    /// Shortest path (BFS, unweighted) between two entities.
    pub fn shortest_path(
        &self,
        source: EntityId,
        target: EntityId,
        max_hops: usize,
    ) -> Result<Option<Path>> {
        if source == target {
            return Ok(Some(Path {
                nodes: vec![source],
                edges: vec![],
                hop_count: 0,
                min_confidence: 1.0,
            }));
        }

        let mut visited: HashSet<EntityId> = HashSet::new();
        // parent map: node → (parent_node, relation_id)
        let mut parent: HashMap<EntityId, (EntityId, RelationId)> = HashMap::new();
        let mut queue: VecDeque<(EntityId, usize)> = VecDeque::new();

        visited.insert(source);
        queue.push_back((source, 0));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_hops {
                continue;
            }

            // Check outgoing edges.
            if let Some(adj) = self.adj_out.get(&current) {
                for (rid, next) in adj.iter() {
                    if !visited.contains(next) {
                        visited.insert(*next);
                        parent.insert(*next, (current, *rid));
                        if *next == target {
                            return Ok(Some(self.reconstruct_path(source, target, &parent)));
                        }
                        queue.push_back((*next, depth + 1));
                    }
                }
            }
            // Also check incoming edges (undirected search).
            if let Some(adj) = self.adj_in.get(&current) {
                for (rid, next) in adj.iter() {
                    if !visited.contains(next) {
                        visited.insert(*next);
                        parent.insert(*next, (current, *rid));
                        if *next == target {
                            return Ok(Some(self.reconstruct_path(source, target, &parent)));
                        }
                        queue.push_back((*next, depth + 1));
                    }
                }
            }
        }

        Ok(None)
    }

    /// All simple paths up to `max_hops` (DFS with backtracking).
    pub fn all_paths(
        &self,
        source: EntityId,
        target: EntityId,
        max_hops: usize,
    ) -> Result<Vec<Path>> {
        let mut results = Vec::new();
        let mut path_nodes = vec![source];
        let mut path_edges = Vec::new();
        let mut visited = HashSet::new();
        visited.insert(source);

        self.dfs_all_paths(
            source,
            target,
            max_hops,
            &mut path_nodes,
            &mut path_edges,
            &mut visited,
            &mut results,
        );

        Ok(results)
    }

    /// Degree centrality for all entities.
    pub fn degree_centrality(&self) -> HashMap<EntityId, f64> {
        let n = self.entities.len();
        if n <= 1 {
            return self.entities.iter().map(|e| (*e.key(), 0.0)).collect();
        }

        let denom = (n - 1) as f64;
        self.entities
            .iter()
            .map(|e| {
                let id = *e.key();
                let out_deg = self.adj_out.get(&id).map_or(0, |a| a.len());
                let in_deg = self.adj_in.get(&id).map_or(0, |a| a.len());
                (id, (out_deg + in_deg) as f64 / denom)
            })
            .collect()
    }

    /// Connected components (undirected).
    pub fn connected_components(&self) -> Vec<Vec<EntityId>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for entry in self.entities.iter() {
            let id = *entry.key();
            if visited.contains(&id) {
                continue;
            }
            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(id);
            visited.insert(id);

            while let Some(current) = queue.pop_front() {
                component.push(current);
                for neighbor in self.raw_neighbor_ids(current, Direction::Both) {
                    if visited.insert(neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }

            components.push(component);
        }

        components
    }

    // ── Internal helpers ─────────────────────────────────────────────

    fn raw_neighbor_ids(&self, id: EntityId, direction: Direction) -> Vec<EntityId> {
        let mut ids = Vec::new();
        if matches!(direction, Direction::Outgoing | Direction::Both) {
            if let Some(adj) = self.adj_out.get(&id) {
                ids.extend(adj.iter().map(|(_, target)| *target));
            }
        }
        if matches!(direction, Direction::Incoming | Direction::Both) {
            if let Some(adj) = self.adj_in.get(&id) {
                ids.extend(adj.iter().map(|(_, source)| *source));
            }
        }
        ids
    }

    fn reconstruct_path(
        &self,
        source: EntityId,
        target: EntityId,
        parent: &HashMap<EntityId, (EntityId, RelationId)>,
    ) -> Path {
        let mut nodes = vec![target];
        let mut edges = Vec::new();
        let mut current = target;
        let mut min_confidence = f64::MAX;

        while current != source {
            let (prev, rid) = parent[&current];
            edges.push(rid);
            nodes.push(prev);
            if let Some(rel) = self.relations.get(&rid) {
                if rel.confidence < min_confidence {
                    min_confidence = rel.confidence;
                }
            }
            current = prev;
        }

        nodes.reverse();
        edges.reverse();

        if min_confidence == f64::MAX {
            min_confidence = 1.0;
        }

        Path {
            hop_count: edges.len(),
            nodes,
            edges,
            min_confidence,
        }
    }

    fn dfs_all_paths(
        &self,
        current: EntityId,
        target: EntityId,
        max_hops: usize,
        path_nodes: &mut Vec<EntityId>,
        path_edges: &mut Vec<RelationId>,
        visited: &mut HashSet<EntityId>,
        results: &mut Vec<Path>,
    ) {
        if path_edges.len() >= max_hops {
            return;
        }

        // Outgoing.
        let out: Vec<_> = self
            .adj_out
            .get(&current)
            .map(|a| a.clone())
            .unwrap_or_default();

        for (rid, next) in out {
            if next == target {
                let mut nodes = path_nodes.clone();
                nodes.push(target);
                let mut edges = path_edges.clone();
                edges.push(rid);
                let min_confidence = edges
                    .iter()
                    .filter_map(|r| self.relations.get(r).map(|rel| rel.confidence))
                    .fold(f64::MAX, f64::min);
                results.push(Path {
                    hop_count: edges.len(),
                    nodes,
                    edges,
                    min_confidence: if min_confidence == f64::MAX {
                        1.0
                    } else {
                        min_confidence
                    },
                });
            } else if !visited.contains(&next) {
                visited.insert(next);
                path_nodes.push(next);
                path_edges.push(rid);
                self.dfs_all_paths(
                    next, target, max_hops, path_nodes, path_edges, visited, results,
                );
                path_nodes.pop();
                path_edges.pop();
                visited.remove(&next);
            }
        }

        // Incoming (for undirected paths).
        let inc: Vec<_> = self
            .adj_in
            .get(&current)
            .map(|a| a.clone())
            .unwrap_or_default();

        for (rid, next) in inc {
            if next == target {
                let mut nodes = path_nodes.clone();
                nodes.push(target);
                let mut edges = path_edges.clone();
                edges.push(rid);
                let min_confidence = edges
                    .iter()
                    .filter_map(|r| self.relations.get(r).map(|rel| rel.confidence))
                    .fold(f64::MAX, f64::min);
                results.push(Path {
                    hop_count: edges.len(),
                    nodes,
                    edges,
                    min_confidence: if min_confidence == f64::MAX {
                        1.0
                    } else {
                        min_confidence
                    },
                });
            } else if !visited.contains(&next) {
                visited.insert(next);
                path_nodes.push(next);
                path_edges.push(rid);
                self.dfs_all_paths(
                    next, target, max_hops, path_nodes, path_edges, visited, results,
                );
                path_nodes.pop();
                path_edges.pop();
                visited.remove(&next);
            }
        }
    }
}

// ── Result types ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Path {
    pub nodes: Vec<EntityId>,
    pub edges: Vec<RelationId>,
    pub hop_count: usize,
    pub min_confidence: f64,
}
