use linked_list::LinkedList;

fn print_list(label: &str, list: &LinkedList<&str>) {
    let parts: Vec<_> = list.iter().map(|v| format!("[{v}]")).collect();
    println!("{label:<10} head ─► {} ─► None", parts.join(" ─► "));
}

fn main() {
    let mut list = LinkedList::new();
    for value in ["c", "b", "a"] {
        list.push_front(value);
    }
    print_list("built:", &list);

    list.reverse();
    print_list("reversed:", &list);

    println!("contains \"b\": {}", list.contains(&"b"));
    println!("len: {}", list.len());

    if let Some(front) = list.pop_front() {
        println!("popped {front:?}");
    }
    print_list("after pop:", &list);
}
