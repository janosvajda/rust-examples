//! # Graph (adjacency list) with BFS and DFS
//!
//! A **graph** is a set of **nodes** (also called vertices) connected by
//! **edges**. Unlike a tree, a graph can have cycles, and there is no root.
//! Road maps, social networks, the web and package dependencies are all graphs.
//!
//! ```text
//!     0 ─── 1 ─── 3
//!     │     │
//!     2 ────┘     4 ─── 5      (two separate groups: {0,1,2,3} and {4,5})
//! ```
//!
//! This example uses an **undirected** graph: every edge goes both ways.
//! Nodes are numbered `0..n`, which keeps the code simple. To use names, keep a
//! separate `HashMap<String, usize>` from names to numbers.
//!
//! ## Storing a graph: the adjacency list
//!
//! For each node, keep a list of its neighbours:
//!
//! ```text
//!   node │ neighbours
//!   ─────┼───────────
//!     0  │ 1, 2
//!     1  │ 0, 3, 2
//!     2  │ 0, 1
//!     3  │ 1
//!     4  │ 5
//!     5  │ 4
//! ```
//!
//! The alternative is an *adjacency matrix*, an n × n grid of true/false. It
//! checks "is there an edge between a and b?" in O(1), but always uses n²
//! memory. Most real graphs have far fewer edges than n², so adjacency lists
//! are the usual choice.
//!
//! ## Two ways to explore a graph
//!
//! | | Breadth-first search (BFS) | Depth-first search (DFS) |
//! |---|---|---|
//! | Idea | Visit all neighbours first, then their neighbours: spreads out in rings | Go as deep as possible along one path, then back up |
//! | Uses a | **queue** (FIFO) | **stack** (LIFO), or recursion |
//! | Good for | Shortest path when edges have no weights | Detecting cycles, topological sort, exploring mazes |
//!
//! Both visit every node and edge once: **O(V + E)**, where V is the number of
//! nodes and E the number of edges. The `visited` list is what stops them
//! looping forever around a cycle.
//!
//! ## Example
//!
//! ```
//! use graph::Graph;
//!
//! let mut graph = Graph::new(4);
//! graph.add_edge(0, 1);
//! graph.add_edge(1, 2);
//! graph.add_edge(2, 3);
//!
//! assert_eq!(graph.bfs(0), vec![0, 1, 2, 3]);
//! assert_eq!(graph.shortest_path(0, 3), Some(vec![0, 1, 2, 3]));
//! ```

use std::collections::VecDeque;

/// An undirected graph with nodes `0..node_count`, stored as an adjacency list.
#[derive(Debug, Clone)]
pub struct Graph {
    // adjacency[n] is the list of nodes connected to node n.
    adjacency: Vec<Vec<usize>>,
}

impl Graph {
    /// Creates a graph with `node_count` nodes and no edges.
    pub fn new(node_count: usize) -> Self {
        Graph {
            adjacency: vec![Vec::new(); node_count],
        }
    }

    /// Returns the number of nodes.
    pub fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Connects `a` and `b`. Because the graph is undirected, the edge is
    /// recorded in both nodes' lists.
    ///
    /// # Panics
    /// Panics if `a` or `b` is not a valid node number.
    pub fn add_edge(&mut self, a: usize, b: usize) {
        self.adjacency[a].push(b);
        self.adjacency[b].push(a);
    }

    /// Returns the neighbours of `node`.
    pub fn neighbours(&self, node: usize) -> &[usize] {
        &self.adjacency[node]
    }

    /// Breadth-first search from `start`. Returns nodes in the order they were visited.
    ///
    /// ```text
    ///   start at 0:  ring 0: [0]   ring 1: [1, 2]   ring 2: [3]
    /// ```
    pub fn bfs(&self, start: usize) -> Vec<usize> {
        let mut visited = vec![false; self.node_count()];
        let mut order = Vec::new();
        let mut queue = VecDeque::new();

        // Mark a node as visited when it is *added* to the queue, not when it
        // is taken out. Otherwise the same node could be queued several times.
        visited[start] = true;
        queue.push_back(start);

        while let Some(node) = queue.pop_front() {
            order.push(node);
            for &neighbour in &self.adjacency[node] {
                if !visited[neighbour] {
                    visited[neighbour] = true;
                    queue.push_back(neighbour);
                }
            }
        }

        order
    }

    /// Depth-first search from `start`, using an explicit stack.
    /// Returns nodes in the order they were visited.
    ///
    /// The code is almost identical to [`Graph::bfs`]. Swapping the queue for
    /// a stack is the only real change, and it turns "wide" into "deep".
    pub fn dfs(&self, start: usize) -> Vec<usize> {
        let mut visited = vec![false; self.node_count()];
        let mut order = Vec::new();
        let mut stack = vec![start];

        while let Some(node) = stack.pop() {
            // A node can be pushed more than once (from different neighbours)
            // before it is visited, so check again here.
            if visited[node] {
                continue;
            }
            visited[node] = true;
            order.push(node);

            // Push neighbours in reverse so the first neighbour ends up on top
            // of the stack and is explored first. That matches the recursive
            // version below.
            for &neighbour in self.adjacency[node].iter().rev() {
                if !visited[neighbour] {
                    stack.push(neighbour);
                }
            }
        }

        order
    }

    /// Depth-first search written recursively. Gives the same order as
    /// [`Graph::dfs`].
    ///
    /// Shorter and closer to how DFS is usually described, but every level of
    /// depth uses a frame on the call stack, so a very deep graph (a long chain
    /// of millions of nodes) can overflow it. The explicit-stack version stores
    /// its stack on the heap and doesn't have that limit.
    pub fn dfs_recursive(&self, start: usize) -> Vec<usize> {
        fn visit(graph: &Graph, node: usize, visited: &mut [bool], order: &mut Vec<usize>) {
            visited[node] = true;
            order.push(node);
            for &neighbour in &graph.adjacency[node] {
                if !visited[neighbour] {
                    visit(graph, neighbour, visited, order);
                }
            }
        }

        let mut visited = vec![false; self.node_count()];
        let mut order = Vec::new();
        visit(self, start, &mut visited, &mut order);
        order
    }

    /// Finds a path from `from` to `to` with the fewest edges, using BFS.
    /// Returns `None` if `to` can't be reached.
    ///
    /// BFS reaches nodes in order of distance, so the *first* time it reaches
    /// a node is via a shortest path. We record which node we came from
    /// (`previous`), then walk those links backwards from `to`.
    pub fn shortest_path(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        let mut previous: Vec<Option<usize>> = vec![None; self.node_count()];
        let mut visited = vec![false; self.node_count()];
        let mut queue = VecDeque::from([from]);
        visited[from] = true;

        while let Some(node) = queue.pop_front() {
            if node == to {
                // Walk back from `to` to `from`, then reverse.
                let mut path = vec![to];
                let mut current = to;
                while let Some(prev) = previous[current] {
                    path.push(prev);
                    current = prev;
                }
                path.reverse();
                return Some(path);
            }
            for &neighbour in &self.adjacency[node] {
                if !visited[neighbour] {
                    visited[neighbour] = true;
                    previous[neighbour] = Some(node);
                    queue.push_back(neighbour);
                }
            }
        }

        None
    }

    /// Splits the graph into its connected groups of nodes.
    ///
    /// Start a BFS from any node not visited yet. Everything it reaches is one
    /// group. Repeat until every node has been visited.
    pub fn connected_components(&self) -> Vec<Vec<usize>> {
        let mut visited = vec![false; self.node_count()];
        let mut components = Vec::new();

        for start in 0..self.node_count() {
            if visited[start] {
                continue;
            }
            let mut component = Vec::new();
            let mut queue = VecDeque::from([start]);
            visited[start] = true;
            while let Some(node) = queue.pop_front() {
                component.push(node);
                for &neighbour in &self.adjacency[node] {
                    if !visited[neighbour] {
                        visited[neighbour] = true;
                        queue.push_back(neighbour);
                    }
                }
            }
            components.push(component);
        }

        components
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The graph from the module docs.
    fn example_graph() -> Graph {
        let mut graph = Graph::new(6);
        graph.add_edge(0, 1);
        graph.add_edge(0, 2);
        graph.add_edge(1, 3);
        graph.add_edge(1, 2);
        graph.add_edge(4, 5);
        graph
    }

    #[test]
    fn neighbours_go_both_ways() {
        let graph = example_graph();
        assert_eq!(graph.neighbours(0), &[1, 2]);
        assert_eq!(graph.neighbours(3), &[1]);
    }

    #[test]
    fn bfs_visits_in_rings() {
        assert_eq!(example_graph().bfs(0), vec![0, 1, 2, 3]);
    }

    #[test]
    fn dfs_goes_deep_first() {
        let graph = example_graph();
        assert_eq!(graph.dfs(0), vec![0, 1, 3, 2]);
        assert_eq!(graph.dfs_recursive(0), vec![0, 1, 3, 2]);
    }

    #[test]
    fn search_stays_inside_the_component() {
        let graph = example_graph();
        assert_eq!(graph.bfs(4), vec![4, 5]);
        assert_eq!(graph.dfs(5), vec![5, 4]);
    }

    #[test]
    fn shortest_path_uses_fewest_edges() {
        let graph = example_graph();
        assert_eq!(graph.shortest_path(2, 3), Some(vec![2, 1, 3]));
        assert_eq!(graph.shortest_path(0, 0), Some(vec![0]));
        assert_eq!(graph.shortest_path(0, 5), None);
    }

    #[test]
    fn finds_connected_components() {
        let components = example_graph().connected_components();
        assert_eq!(components, vec![vec![0, 1, 2, 3], vec![4, 5]]);
    }

    #[test]
    fn handles_cycles() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_edge(2, 0);
        assert_eq!(graph.bfs(0).len(), 3);
        assert_eq!(graph.dfs(0).len(), 3);
    }

    #[test]
    fn iterative_dfs_handles_a_very_long_chain() {
        let n = 1_000_000;
        let mut graph = Graph::new(n);
        for i in 0..n - 1 {
            graph.add_edge(i, i + 1);
        }
        assert_eq!(graph.dfs(0).len(), n);
    }
}
