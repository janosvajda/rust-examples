//! # Weighted graph and Dijkstra's shortest path
//!
//! In a **weighted** graph every edge has a cost: a distance, a travel time, a
//! price. The shortest path is now the one with the **lowest total cost**, not
//! the one with the fewest edges. Breadth-first search no longer works:
//!
//! ```text
//!           A
//!         /   \
//!      10/     \1        BFS picks A → B (1 edge, cost 10).
//!       /       \        The cheapest path is A → C → D → B (3 edges, cost 3).
//!      B         C
//!       \       /
//!       1\     /1
//!         \   /
//!           D
//! ```
//!
//! ## Dijkstra's algorithm
//!
//! 1. Set every node's distance to infinity, except the start, which is 0.
//! 2. Repeatedly pick the **unvisited node with the smallest distance**. Its
//!    distance is now final, because every other route to it would go through
//!    a node that is already further away, and costs are never negative.
//! 3. For each neighbour, check whether going through this node is cheaper
//!    than what we know so far. If so, update the neighbour's distance. This
//!    is called *relaxing* the edge.
//! 4. Stop when every reachable node is final.
//!
//! Step 2 is where a **min-heap** (priority queue) comes in: it hands back the
//! closest node in O(log n) instead of scanning every node. See the
//! `heaps/binary-heap` example for how a heap works.
//!
//! **Requirement:** edge weights must not be negative. With negative weights a
//! "final" distance could still get cheaper later. (The Bellman-Ford
//! algorithm handles that case.)
//!
//! ## Complexity
//!
//! **O(V + E log(E + 1))** for this lazy binary heap, where V counts nodes
//! and E counts edges, including parallel edges. For simple graphs this is
//! commonly written O((V + E) log V).
//!
//! ## Example
//!
//! ```
//! use dijkstra::WeightedGraph;
//!
//! // The graph from the diagram: A=0, B=1, C=2, D=3.
//! let mut graph = WeightedGraph::new(4);
//! graph.add_edge(0, 1, 10);
//! graph.add_edge(0, 2, 1);
//! graph.add_edge(2, 3, 1);
//! graph.add_edge(3, 1, 1);
//!
//! let (cost, path) = graph.shortest_path(0, 1).unwrap().unwrap();
//! assert_eq!(cost, 3);              // A → C → D → B
//! assert_eq!(path, vec![0, 2, 3, 1]);
//! ```

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    InvalidNode { node: usize, node_count: usize },
    CostOverflow,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNode { node, node_count } => {
                write!(f, "node {node} is outside 0..{node_count}")
            }
            Self::CostOverflow => write!(f, "a route cost exceeds u64::MAX"),
        }
    }
}

impl std::error::Error for PathError {}

/// Distances and predecessor nodes, both indexed by node number.
pub type SearchResult = (Vec<Option<u64>>, Vec<Option<usize>>);

/// An edge to `to` with the given `cost`.
#[derive(Debug, Clone, Copy)]
struct Edge {
    to: usize,
    cost: u32,
}

/// An undirected graph with nodes `0..node_count` and non-negative edge costs.
///
/// Costs are `u32`, an unsigned type, so the compiler itself guarantees
/// they can't be negative, which Dijkstra requires.
#[derive(Debug, Clone)]
pub struct WeightedGraph {
    adjacency: Vec<Vec<Edge>>,
}

impl WeightedGraph {
    /// Creates a graph with `node_count` nodes and no edges.
    pub fn new(node_count: usize) -> Self {
        WeightedGraph {
            adjacency: vec![Vec::new(); node_count],
        }
    }

    /// Returns the number of nodes.
    pub fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Connects `a` and `b` in both directions with the given `cost`.
    pub fn add_edge(&mut self, a: usize, b: usize, cost: u32) {
        assert!(
            a < self.node_count() && b < self.node_count(),
            "both endpoints must exist"
        );
        self.adjacency[a].push(Edge { to: b, cost });
        self.adjacency[b].push(Edge { to: a, cost });
    }

    /// Runs Dijkstra from `start`. Totals use u64 even though individual edges use u32.
    /// Returns an error for an invalid start or a total that overflows u64.
    ///
    /// Returns two lists, indexed by node:
    /// - `distances[n]`: the lowest total cost from `start` to `n`, or `None`
    ///   if `n` is unreachable.
    /// - `previous[n]`: the node before `n` on that cheapest path. Follow
    ///   these links back to `start` to rebuild the path.
    pub fn dijkstra(&self, start: usize) -> Result<SearchResult, PathError> {
        let n = self.node_count();
        if start >= n {
            return Err(PathError::InvalidNode {
                node: start,
                node_count: n,
            });
        }
        let mut distances: Vec<Option<u64>> = vec![None; n]; // None = infinity
        let mut previous: Vec<Option<usize>> = vec![None; n];

        // std's BinaryHeap is a *max*-heap. Wrapping the entries in `Reverse`
        // flips the ordering, so `pop` returns the *smallest* distance.
        // Tuples compare by their first field first, so the heap orders by distance.
        let mut heap = BinaryHeap::new();

        distances[start] = Some(0);
        heap.push(Reverse((0u64, start)));

        while let Some(Reverse((distance, node))) = heap.pop() {
            // We never remove old entries when a node's distance improves. We
            // just push a new, cheaper entry. So when an outdated entry comes
            // out of the heap later, skip it. (This "lazy deletion" is simpler
            // than updating entries in place.)
            if distances[node].is_some_and(|best| distance > best) {
                continue;
            }

            for edge in &self.adjacency[node] {
                let new_distance = distance
                    .checked_add(u64::from(edge.cost))
                    .ok_or(PathError::CostOverflow)?;
                // Relax the edge: is going through `node` cheaper?
                let is_better = match distances[edge.to] {
                    None => true,
                    Some(current) => new_distance < current,
                };
                if is_better {
                    distances[edge.to] = Some(new_distance);
                    previous[edge.to] = Some(node);
                    heap.push(Reverse((new_distance, edge.to)));
                }
            }
        }

        Ok((distances, previous))
    }

    /// Returns the total cost and the list of nodes on the cheapest path from
    /// `from` to `to`, or `Ok(None)` if `to` is unreachable.
    /// Invalid nodes and unrepresentable costs return an error.
    pub fn shortest_path(
        &self,
        from: usize,
        to: usize,
    ) -> Result<Option<(u64, Vec<usize>)>, PathError> {
        if to >= self.node_count() {
            return Err(PathError::InvalidNode {
                node: to,
                node_count: self.node_count(),
            });
        }
        let (distances, previous) = self.dijkstra(from)?;
        let Some(cost) = distances[to] else {
            return Ok(None);
        };

        let mut path = vec![to];
        let mut current = to;
        while let Some(prev) = previous[current] {
            path.push(prev);
            current = prev;
        }
        path.reverse();

        Ok(Some((cost, path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The graph from the module docs: A=0, B=1, C=2, D=3.
    fn example_graph() -> WeightedGraph {
        let mut graph = WeightedGraph::new(4);
        graph.add_edge(0, 1, 10);
        graph.add_edge(0, 2, 1);
        graph.add_edge(2, 3, 1);
        graph.add_edge(3, 1, 1);
        graph
    }

    #[test]
    fn prefers_cheaper_path_over_fewer_edges() {
        let mut graph = WeightedGraph::new(3);
        graph.add_edge(0, 2, 10);
        graph.add_edge(0, 1, 2);
        graph.add_edge(1, 2, 3);
        assert_eq!(graph.shortest_path(0, 2).unwrap(), Some((5, vec![0, 1, 2])));
    }

    #[test]
    fn distances_to_all_nodes() {
        let (distances, _) = example_graph().dijkstra(0).unwrap();
        assert_eq!(distances, vec![Some(0), Some(3), Some(1), Some(2)]);
    }

    #[test]
    fn path_to_start_is_just_start() {
        assert_eq!(
            example_graph().shortest_path(2, 2).unwrap(),
            Some((0, vec![2]))
        );
    }

    #[test]
    fn unreachable_node() {
        let mut graph = WeightedGraph::new(3);
        graph.add_edge(0, 1, 1);
        assert_eq!(graph.shortest_path(0, 2).unwrap(), None);
        let (distances, _) = graph.dijkstra(0).unwrap();
        assert_eq!(distances[2], None);
    }

    #[test]
    fn zero_cost_edges_are_allowed() {
        let mut graph = WeightedGraph::new(3);
        graph.add_edge(0, 1, 0);
        graph.add_edge(1, 2, 0);
        assert_eq!(graph.shortest_path(0, 2).unwrap(), Some((0, vec![0, 1, 2])));
    }
    #[test]
    fn costs_larger_than_u32_do_not_wrap_or_create_a_predecessor_cycle() {
        let mut graph = WeightedGraph::new(3);
        graph.add_edge(0, 1, u32::MAX);
        graph.add_edge(1, 2, 1);
        assert_eq!(
            graph.shortest_path(0, 2),
            Ok(Some((u64::from(u32::MAX) + 1, vec![0, 1, 2])))
        );
        assert!(matches!(
            graph.dijkstra(3),
            Err(PathError::InvalidNode { .. })
        ));
        assert!(matches!(
            graph.shortest_path(0, 3),
            Err(PathError::InvalidNode { .. })
        ));
        assert!(WeightedGraph::new(0).dijkstra(0).is_err());
    }
}
