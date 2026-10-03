//! # Topological sort (dependency ordering)
//!
//! Some tasks must happen before others: a crate must be compiled before the
//! crates that depend on it, and a course must be taken before the courses
//! that require it. A **topological sort** puts the tasks in an order where
//! every task comes **after everything it depends on**.
//!
//! ## Directed graphs
//!
//! The graph examples so far were *undirected*: friendship goes both ways. Here
//! edges have a **direction**. An edge `a → b` means "a must come before b".
//!
//! ```text
//!   serde ──────► serde_json ──────► my_app
//!                                      ▲
//!   rand ───────────────────────────────┘
//!     │
//!     └─────────► uuid ────────────► my_app
//!
//!   Valid orders: serde, rand, serde_json, uuid, my_app
//!                 rand, uuid, serde, serde_json, my_app   …and others
//! ```
//!
//! There's usually more than one valid order. There's **no** valid order if
//! the graph has a **cycle** (a → b → c → a), because then every task in the
//! cycle has to come before itself. A directed graph without cycles is called
//! a **DAG** (directed acyclic graph).
//!
//! ## Kahn's algorithm
//!
//! 1. Count each node's **in-degree**: how many edges point *into* it, which
//!    is how many things it still waits for.
//! 2. Put every node with in-degree 0 (nothing to wait for) in a queue.
//! 3. Take a node from the queue and add it to the result. For each edge
//!    leaving it, decrease that neighbour's in-degree. If it drops to 0,
//!    the neighbour is ready, so queue it.
//! 4. If the result has every node, done. If some nodes are left over, they
//!    are waiting on each other: there's a cycle.
//!
//! Each node and edge is handled once: **O(V + E)**.
//!
//! ## Finding the actual cycle
//!
//! Kahn's algorithm says *that* there's a cycle, not *where*. To report it,
//! [`DirectedGraph::find_cycle`] uses depth-first search with three colours:
//!
//! - **White**: not visited yet.
//! - **Grey**: visit in progress, so the node is on the current DFS path.
//! - **Black**: finished, everything reachable from it has been explored.
//!
//! Reaching a **grey** node means we've looped back to a node on our own
//! path, and the path from that node to here is the cycle.
//!
//! ## Example
//!
//! ```
//! use topological_sort::DirectedGraph;
//!
//! // 0 = shirt, 1 = tie, 2 = jacket
//! let mut graph = DirectedGraph::new(3);
//! graph.add_edge(0, 1); // shirt before tie
//! graph.add_edge(1, 2); // tie before jacket
//! graph.add_edge(0, 2); // shirt before jacket
//!
//! assert_eq!(graph.topological_sort(), Ok(vec![0, 1, 2]));
//!
//! graph.add_edge(2, 0); // jacket before shirt: impossible!
//! assert!(graph.topological_sort().is_err());
//! assert_eq!(graph.find_cycle(), Some(vec![0, 1, 2, 0]));
//! ```

use std::collections::VecDeque;

/// A directed graph with nodes `0..node_count`.
#[derive(Debug, Clone)]
pub struct DirectedGraph {
    // edges[a] lists every b with an edge a → b.
    edges: Vec<Vec<usize>>,
}

/// Returned by [`DirectedGraph::topological_sort`] when the graph has a cycle.
#[derive(Debug, PartialEq, Eq)]
pub struct CycleError {
    /// The nodes that could not be ordered: they are on a cycle, or depend on one.
    pub unordered: Vec<usize>,
}

/// The three DFS states used by [`DirectedGraph::find_cycle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Colour {
    White,
    Grey,
    Black,
}

impl DirectedGraph {
    /// Creates a graph with `node_count` nodes and no edges.
    pub fn new(node_count: usize) -> Self {
        DirectedGraph {
            edges: vec![Vec::new(); node_count],
        }
    }

    /// Returns the number of nodes.
    pub fn node_count(&self) -> usize {
        self.edges.len()
    }

    /// Adds the edge `from → to`, meaning "`from` must come before `to`".
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges[from].push(to);
    }

    /// Orders the nodes so that every edge points forward, using Kahn's
    /// algorithm. Returns a [`CycleError`] if that's impossible.
    ///
    /// When several nodes are ready at once, they come out in the order they
    /// became ready (lowest number first at the start).
    pub fn topological_sort(&self) -> Result<Vec<usize>, CycleError> {
        let n = self.node_count();

        // Step 1: count incoming edges.
        let mut in_degree = vec![0; n];
        for targets in &self.edges {
            for &to in targets {
                in_degree[to] += 1;
            }
        }

        // Step 2: everything that waits for nothing is ready.
        let mut ready: VecDeque<usize> = (0..n).filter(|&node| in_degree[node] == 0).collect();

        // Step 3: take ready nodes, and release whatever was waiting on them.
        let mut order = Vec::with_capacity(n);
        while let Some(node) = ready.pop_front() {
            order.push(node);
            for &next in &self.edges[node] {
                in_degree[next] -= 1;
                if in_degree[next] == 0 {
                    ready.push_back(next);
                }
            }
        }

        // Step 4: leftovers mean a cycle.
        if order.len() == n {
            Ok(order)
        } else {
            let unordered = (0..n).filter(|&node| in_degree[node] > 0).collect();
            Err(CycleError { unordered })
        }
    }

    /// Returns one cycle as a list of nodes that starts and ends with the same
    /// node, like `[2, 5, 3, 2]`. Returns `None` if the graph has no cycle.
    pub fn find_cycle(&self) -> Option<Vec<usize>> {
        let mut colour = vec![Colour::White; self.node_count()];
        let mut path = Vec::new();

        for start in 0..self.node_count() {
            if colour[start] == Colour::White
                && let Some(cycle) = self.dfs_find_cycle(start, &mut colour, &mut path)
            {
                return Some(cycle);
            }
        }
        None
    }

    /// Recursive DFS for [`DirectedGraph::find_cycle`]. `path` holds the grey
    /// nodes, in the order they were entered.
    fn dfs_find_cycle(
        &self,
        node: usize,
        colour: &mut [Colour],
        path: &mut Vec<usize>,
    ) -> Option<Vec<usize>> {
        colour[node] = Colour::Grey;
        path.push(node);

        for &next in &self.edges[node] {
            match colour[next] {
                Colour::Grey => {
                    // `next` is on the current path: we've found a loop.
                    let start = path.iter().position(|&n| n == next).expect("grey nodes are on the path");
                    let mut cycle = path[start..].to_vec();
                    cycle.push(next);
                    return Some(cycle);
                }
                Colour::White => {
                    if let Some(cycle) = self.dfs_find_cycle(next, colour, path) {
                        return Some(cycle);
                    }
                }
                Colour::Black => {} // already fully explored, no cycle through it
            }
        }

        path.pop();
        colour[node] = Colour::Black;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks that every edge goes from an earlier node to a later one.
    fn respects_every_edge(graph: &DirectedGraph, order: &[usize]) -> bool {
        let mut position = vec![0; graph.node_count()];
        for (i, &node) in order.iter().enumerate() {
            position[node] = i;
        }
        graph
            .edges
            .iter()
            .enumerate()
            .all(|(from, targets)| targets.iter().all(|&to| position[from] < position[to]))
    }

    #[test]
    fn sorts_a_dag() {
        // The crate graph from the module docs:
        // 0 serde, 1 serde_json, 2 rand, 3 uuid, 4 my_app
        let mut graph = DirectedGraph::new(5);
        graph.add_edge(0, 1);
        graph.add_edge(1, 4);
        graph.add_edge(2, 4);
        graph.add_edge(2, 3);
        graph.add_edge(3, 4);

        let order = graph.topological_sort().unwrap();
        assert_eq!(order, vec![0, 2, 1, 3, 4]);
        assert!(respects_every_edge(&graph, &order));
        assert_eq!(graph.find_cycle(), None);
    }

    #[test]
    fn nodes_without_edges_are_included() {
        let graph = DirectedGraph::new(3);
        assert_eq!(graph.topological_sort(), Ok(vec![0, 1, 2]));
    }

    #[test]
    fn detects_a_cycle() {
        // 0 → 1 → 2 → 3 → 1, and 3 → 4
        let mut graph = DirectedGraph::new(5);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 1);
        graph.add_edge(3, 4);

        // 0 can be ordered; 1, 2, 3 are the cycle; 4 depends on it.
        assert_eq!(
            graph.topological_sort(),
            Err(CycleError { unordered: vec![1, 2, 3, 4] })
        );
        assert_eq!(graph.find_cycle(), Some(vec![1, 2, 3, 1]));
    }

    #[test]
    fn self_loop_is_a_cycle() {
        let mut graph = DirectedGraph::new(2);
        graph.add_edge(0, 1);
        graph.add_edge(1, 1);
        assert!(graph.topological_sort().is_err());
        assert_eq!(graph.find_cycle(), Some(vec![1, 1]));
    }

    #[test]
    fn diamond_shape_is_not_a_cycle() {
        // 0 → 1 → 3 and 0 → 2 → 3: node 3 is reached twice, but there's no loop.
        // DFS meets 3 again as a *black* node, which is fine.
        let mut graph = DirectedGraph::new(4);
        graph.add_edge(0, 1);
        graph.add_edge(0, 2);
        graph.add_edge(1, 3);
        graph.add_edge(2, 3);
        assert_eq!(graph.find_cycle(), None);
        assert!(graph.topological_sort().is_ok());
    }
}
