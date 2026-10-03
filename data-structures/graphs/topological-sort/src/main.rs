use topological_sort::DirectedGraph;

fn main() {
    // Which order should Cargo build these crates in?
    // An edge `a → b` means "b depends on a", so a must be built first.
    let crates = ["serde", "serde_json", "rand", "uuid", "tokio", "my_app"];
    let dependencies = [
        ("serde", "serde_json"),
        ("serde_json", "my_app"),
        ("rand", "uuid"),
        ("uuid", "my_app"),
        ("tokio", "my_app"),
    ];

    let index_of = |name: &str| crates.iter().position(|&c| c == name).unwrap();
    let mut graph = DirectedGraph::new(crates.len());
    for (dependency, dependent) in dependencies {
        graph.add_edge(index_of(dependency), index_of(dependent));
    }

    match graph.topological_sort() {
        Ok(order) => {
            println!("build order:");
            for (step, node) in order.iter().enumerate() {
                println!("  {}. {}", step + 1, crates[*node]);
            }
        }
        Err(error) => println!("cycle! could not order: {:?}", error.unordered),
    }

    println!("\nnow add a circular dependency: my_app → serde");
    graph.add_edge(index_of("my_app"), index_of("serde"));

    if let Err(error) = graph.topological_sort() {
        let names: Vec<&str> = error.unordered.iter().map(|&n| crates[n]).collect();
        println!("cannot build, these crates are stuck: {names:?}");
    }
    if let Some(cycle) = graph.find_cycle() {
        let names: Vec<&str> = cycle.iter().map(|&n| crates[n]).collect();
        println!("the cycle: {}", names.join(" → "));
    }
}
