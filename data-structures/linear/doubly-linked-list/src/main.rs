use doubly_linked_list::DoublyLinkedList;

fn main() {
    // A music playlist: you can add songs at either end and play from either end.
    let mut playlist = DoublyLinkedList::new();
    playlist.push_back("Song B");
    playlist.push_back("Song C");
    playlist.push_front("Song A"); // jump the queue

    println!("forward:  {:?}", playlist.to_vec());
    println!("backward: {:?}", playlist.to_vec_reversed());

    if let Some(first) = playlist.peek_front() {
        println!("\nfirst song: {}", *first);
    } // the `Ref` guard returned by peek_front is dropped here

    println!("\nplay from the end: {:?}", playlist.pop_back());
    println!("play from the start: {:?}", playlist.pop_front());
    println!("left: {:?} (len = {})", playlist.to_vec(), playlist.len());
}
