use binary_heap::{MinHeap, heap_sort};

fn main() {
    println!("--- Priority queue ---");
    // Tuples compare by their first element first, so the lowest number is the
    // most urgent task.
    let mut tasks = MinHeap::new();
    tasks.push((3, "refactor the parser"));
    tasks.push((1, "fix production bug"));
    tasks.push((2, "review pull request"));
    tasks.push((1, "answer the pager"));

    while let Some((priority, task)) = tasks.pop() {
        println!("priority {priority}: {task}");
    }

    println!("\n--- Heap sort ---");
    let values = vec![42, 7, 19, 3, 25, 11, 3];
    println!("input:  {values:?}");
    println!("sorted: {:?}", heap_sort(values));
}
