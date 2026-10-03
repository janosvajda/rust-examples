use graph::Graph;

fn main() {
    // A small social network. Each edge means "these two people are friends".
    let names = ["Anna", "Bence", "Csilla", "Dani", "Eszter", "Feri", "Gabi"];
    let mut graph = Graph::new(names.len());
    let friendships = [(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (5, 6)];
    for (a, b) in friendships {
        graph.add_edge(a, b);
    }

    let to_names = |nodes: &[usize]| -> Vec<&str> { nodes.iter().map(|&n| names[n]).collect() };

    println!("BFS from Anna: {:?}", to_names(&graph.bfs(0)));
    println!("DFS from Anna: {:?}", to_names(&graph.dfs(0)));

    println!();
    match graph.shortest_path(0, 4) {
        Some(path) => println!(
            "Anna to Eszter: {} ({} handshakes)",
            to_names(&path).join(" → "),
            path.len() - 1
        ),
        None => println!("Anna and Eszter are not connected"),
    }
    if graph.shortest_path(0, 6).is_none() {
        println!("Anna and Gabi are not connected");
    }

    println!("\nFriend groups:");
    for group in graph.connected_components() {
        println!("  {:?}", to_names(&group));
    }
}
