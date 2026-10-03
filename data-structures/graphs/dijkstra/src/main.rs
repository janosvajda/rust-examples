use dijkstra::WeightedGraph;

fn main() {
    // Driving times in minutes between some Hungarian cities (rounded, for illustration).
    let cities = ["Budapest", "Győr", "Székesfehérvár", "Veszprém", "Kecskemét", "Szeged", "Pécs"];
    let roads = [
        (0, 1, 80),  // Budapest – Győr
        (0, 2, 50),  // Budapest – Székesfehérvár
        (2, 3, 40),  // Székesfehérvár – Veszprém
        (1, 3, 70),  // Győr – Veszprém
        (0, 4, 60),  // Budapest – Kecskemét
        (4, 5, 60),  // Kecskemét – Szeged
        (2, 6, 140), // Székesfehérvár – Pécs
        (5, 6, 150), // Szeged – Pécs
    ];

    let mut graph = WeightedGraph::new(cities.len());
    for (a, b, minutes) in roads {
        graph.add_edge(a, b, minutes);
    }

    println!("Fastest routes from Budapest:");
    for (destination, name) in cities.iter().enumerate().skip(1) {
        if let Some((minutes, path)) = graph.shortest_path(0, destination) {
            let route: Vec<&str> = path.iter().map(|&n| cities[n]).collect();
            println!("  {name:<15} {minutes:>3} min  via {}", route.join(" → "));
        }
    }
}
