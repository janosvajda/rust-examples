use union_find::UnionFind;

/// Kruskal's algorithm: find the cheapest set of cables that connects every
/// office (a *minimum spanning tree*).
///
/// 1. Sort all possible cables from cheapest to most expensive.
/// 2. Take each cable in turn. If its two offices are already connected,
///    skip it, because it would only form a loop. Otherwise lay it, and merge
///    the two offices' groups.
///
/// Union-find makes step 2 almost free: `union(a, b)` merges the two groups
/// and returns `false` if `a` and `b` were already connected.
fn main() {
    let offices = ["Budapest", "Debrecen", "Szeged", "Pécs", "Győr"];
    // (cost in millions, office a, office b)
    let mut cables = vec![
        (12, 0, 1),
        (5, 0, 2),
        (3, 1, 2),
        (9, 2, 3),
        (6, 0, 4),
        (15, 3, 4),
        (11, 1, 4),
        (4, 0, 3),
    ];
    cables.sort();

    let mut network = UnionFind::new(offices.len());
    let mut total_cost = 0;

    for (cost, a, b) in cables {
        if network.union(a, b) {
            total_cost += cost;
            println!("lay   {:>9} – {:<9} cost {cost:>2}", offices[a], offices[b]);
        } else {
            println!("skip  {:>9} – {:<9} (already connected)", offices[a], offices[b]);
        }
    }

    println!("\nall offices connected: {}", network.group_count() == 1);
    println!("total cost: {total_cost} million");
}
