use queue::Queue;

fn main() {
    // A print queue: documents are printed in the order they were sent.
    let mut print_queue = Queue::new();

    for document in [
        "report.pdf",
        "photo.jpg",
        "invoice.pdf",
        "notes.txt",
        "slides.pdf",
    ] {
        print_queue.enqueue(document);
        println!(
            "queued {document:<12} (len = {}, capacity = {})",
            print_queue.len(),
            print_queue.capacity()
        );
    }

    if let Some(next) = print_queue.peek() {
        println!("\nnext to print: {next}");
    }

    while let Some(document) = print_queue.dequeue() {
        println!("printing {document}");
    }
}
