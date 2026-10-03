// Lesson 4: channels and shared state.
//
// Tasks need to communicate. Two approaches:
//   channels      send messages between tasks (mpsc: many → one, oneshot: one reply)
//   shared state  Arc<Mutex<T>>, with the same rules as threads
// "Share memory by communicating" (channels) is often the simpler design.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::time::sleep;

// ---- 1. mpsc: many producers, one consumer -----------------------------------------

#[derive(Debug)]
enum Event {
    Order { table: u32, dish: &'static str },
    Closed { waiter: &'static str },
}

/// The kitchen receives orders from every waiter through one channel.
async fn kitchen(mut orders: mpsc::Receiver<Event>) -> Vec<String> {
    let mut cooked = Vec::new();
    // `recv()` waits for the next message; it returns None once every
    // sender has been dropped and the channel is empty.
    while let Some(event) = orders.recv().await {
        match event {
            Event::Order { table, dish } => cooked.push(format!("{dish} for table {table}")),
            Event::Closed { waiter } => println!("    {waiter} has finished their shift"),
        }
    }
    cooked
}

async fn waiter(name: &'static str, tables: Vec<(u32, &'static str)>, orders: mpsc::Sender<Event>) {
    for (table, dish) in tables {
        sleep(Duration::from_millis(50)).await;
        // `send` waits if the channel is full: built-in backpressure.
        orders.send(Event::Order { table, dish }).await.unwrap();
    }
    orders.send(Event::Closed { waiter: name }).await.unwrap();
} // `orders` (this waiter's sender) is dropped here

// ---- 2. oneshot: ask a question, get exactly one answer ----------------------------

/// A request carries its own reply channel.
struct PriceRequest {
    item: String,
    reply: oneshot::Sender<Option<u32>>,
}

async fn price_service(mut requests: mpsc::Receiver<PriceRequest>) {
    while let Some(request) = requests.recv().await {
        let price = match request.item.as_str() {
            "coffee" => Some(250),
            "cake" => Some(400),
            _ => None,
        };
        let _ = request.reply.send(price); // the asker may have given up: ignore that
    }
}

async fn ask_price(service: &mpsc::Sender<PriceRequest>, item: &str) -> Option<u32> {
    let (reply, answer) = oneshot::channel();
    service.send(PriceRequest { item: item.to_string(), reply }).await.ok()?;
    answer.await.ok()?
}

// ---- 3. Shared state: Arc<Mutex<T>> --------------------------------------------------

/// Several tasks update one counter. `tokio::sync::Mutex` can be held across
/// an `.await`; `lock().await` waits for its turn without blocking a thread.
async fn count_visits(tasks: u32, visits_each: u32) -> u32 {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();
    for _ in 0..tasks {
        let counter = Arc::clone(&counter);
        handles.push(tokio::spawn(async move {
            for _ in 0..visits_each {
                let mut count = counter.lock().await;
                *count += 1;
            } // the guard is dropped at the end of each loop iteration: lock released
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }
    *counter.lock().await
}

#[tokio::main]
async fn main() {
    println!("1. mpsc: two waiters, one kitchen");
    let (sender, receiver) = mpsc::channel(8); // buffer of 8 messages
    let kitchen_task = tokio::spawn(kitchen(receiver));
    let anna = tokio::spawn(waiter("Anna", vec![(1, "soup"), (3, "pasta")], sender.clone()));
    let ben = tokio::spawn(waiter("Ben", vec![(2, "salad")], sender));
    // Both waiters own a Sender. When both finish, the kitchen's loop ends.
    anna.await.unwrap();
    ben.await.unwrap();
    for dish in kitchen_task.await.unwrap() {
        println!("    cooked: {dish}");
    }

    println!("\n2. oneshot: request and reply");
    let (service, requests) = mpsc::channel(4);
    tokio::spawn(price_service(requests));
    for item in ["coffee", "cake", "yacht"] {
        println!("    price of {item}: {:?}", ask_price(&service, item).await);
    }

    println!("\n3. Shared state with Arc<Mutex<T>>");
    println!("    10 tasks × 100 visits = {}", count_visits(10, 100).await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn kitchen_receives_every_order_then_stops() {
        let (sender, receiver) = mpsc::channel(2);
        let kitchen_task = tokio::spawn(kitchen(receiver));
        waiter("A", vec![(1, "tea"), (2, "toast")], sender).await;
        assert_eq!(kitchen_task.await.unwrap(), ["tea for table 1", "toast for table 2"]);
    }

    #[tokio::test]
    async fn oneshot_reply() {
        let (service, requests) = mpsc::channel(1);
        tokio::spawn(price_service(requests));
        assert_eq!(ask_price(&service, "cake").await, Some(400));
        assert_eq!(ask_price(&service, "boat").await, None);
    }

    #[tokio::test]
    async fn no_increments_are_lost() {
        assert_eq!(count_visits(8, 250).await, 2000);
    }

    #[tokio::test]
    async fn recv_returns_none_when_all_senders_are_gone() {
        let (sender, mut receiver) = mpsc::channel::<u8>(1);
        drop(sender);
        assert_eq!(receiver.recv().await, None);
    }
}
