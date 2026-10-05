// Lesson 4: channels and shared state.
//
// Tasks need to communicate. Two approaches:
//   channels      send messages between tasks (mpsc: many → one, oneshot: one reply)
//   shared state  Arc<Mutex<T>>, with the same rules as threads
// A service task can own its data and process requests through channels.

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
    // strong sender has been dropped and the channel is drained. Explicit
    // Receiver::close() also closes it; reserved slots must be released/drained.
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

// ---- 2. oneshot: ask a question, receive at most one answer -----------------------

/// A request carries its own reply channel.
struct PriceRequest {
    item: String,
    reply: oneshot::Sender<Option<u32>>,
}

#[derive(Debug, PartialEq, Eq)]
enum PriceError {
    ServiceClosed,
    ReplyDropped,
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

/// Ok(Some(price)) is a price in cents; Ok(None) is an unknown item.
/// Channel failures are separate errors instead of being mistaken for unknown items.
async fn ask_price(
    service: &mpsc::Sender<PriceRequest>,
    item: &str,
) -> Result<Option<u32>, PriceError> {
    let (reply, answer) = oneshot::channel();
    service
        .send(PriceRequest {
            item: item.to_string(),
            reply,
        })
        .await
        .map_err(|_| PriceError::ServiceClosed)?;
    answer.await.map_err(|_| PriceError::ReplyDropped)
}

// ---- 3. Shared state: Arc<Mutex<T>> --------------------------------------------------

/// Several tasks update one counter. `tokio::sync::Mutex` can be held across
/// an `.await`; `lock().await` waits for its turn without blocking a thread.
/// The maximum product of two u32 inputs fits in u64. Demo inputs are small;
/// large workloads need a task limit, as shown in lesson 2.
async fn count_visits(tasks: u32, visits_each: u32) -> u64 {
    let counter = Arc::new(Mutex::new(0_u64));
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
    let anna = tokio::spawn(waiter(
        "Anna",
        vec![(1, "soup"), (3, "pasta")],
        sender.clone(),
    ));
    let ben = tokio::spawn(waiter("Ben", vec![(2, "salad")], sender));
    // Both waiters own a Sender. When both finish, the kitchen's loop ends.
    anna.await.unwrap();
    ben.await.unwrap();
    for dish in kitchen_task.await.unwrap() {
        println!("    cooked: {dish}");
    }

    println!("\n2. oneshot: request and reply");
    let (service, requests) = mpsc::channel(4);
    let service_task = tokio::spawn(price_service(requests));
    for item in ["coffee", "cake", "yacht"] {
        println!("    price of {item}: {:?}", ask_price(&service, item).await);
    }
    drop(service); // no more requests: drain the queue and end the service loop
    service_task.await.unwrap();

    println!("\n3. Shared state with Arc<Mutex<T>>");
    println!(
        "    10 tasks × 100 visits = {}",
        count_visits(10, 100).await
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn kitchen_receives_every_order_then_stops() {
        let (sender, receiver) = mpsc::channel(2);
        let kitchen_task = tokio::spawn(kitchen(receiver));
        waiter("A", vec![(1, "tea"), (2, "toast")], sender).await;
        assert_eq!(
            kitchen_task.await.unwrap(),
            ["tea for table 1", "toast for table 2"]
        );
    }

    #[tokio::test]
    async fn oneshot_reply() {
        let (service, requests) = mpsc::channel(1);
        let task = tokio::spawn(price_service(requests));
        assert_eq!(ask_price(&service, "cake").await, Ok(Some(400)));
        assert_eq!(ask_price(&service, "boat").await, Ok(None));
        drop(service);
        task.await.unwrap();
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

    #[tokio::test]
    async fn a_closed_service_is_an_error_not_an_unknown_item() {
        let (service, requests) = mpsc::channel(1);
        drop(requests);
        assert_eq!(
            ask_price(&service, "cake").await,
            Err(PriceError::ServiceClosed)
        );
    }

    #[tokio::test]
    async fn a_dropped_reply_is_an_error_not_an_unknown_item() {
        let (service, mut requests) = mpsc::channel::<PriceRequest>(1);
        let task = tokio::spawn(async move {
            let request = requests.recv().await.unwrap();
            drop(request.reply);
        });
        assert_eq!(
            ask_price(&service, "cake").await,
            Err(PriceError::ReplyDropped)
        );
        task.await.unwrap();
    }

    #[tokio::test]
    async fn closing_a_receiver_still_allows_queued_messages_to_be_drained() {
        let (sender, mut receiver) = mpsc::channel(1);
        sender.send(7).await.unwrap();
        receiver.close(); // a sender still exists
        assert!(sender.send(8).await.is_err());
        assert_eq!(receiver.recv().await, Some(7));
        assert_eq!(receiver.recv().await, None);
    }
}
