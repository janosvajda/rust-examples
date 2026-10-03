use std::sync::{Arc, Mutex};
use std::thread;

// ---------- Box<T> ----------
// A Box owns a value on the heap. The Box itself is just a pointer on the stack,
// and the heap memory is freed when the Box goes out of scope.

// 1. Recursive types.
// Without a Box this enum would have infinite size, because an `Expr` would contain an `Expr`.
// A Box has a fixed size (one pointer), so the compiler can lay out the type.
#[derive(Debug)]
enum Expr {
    Num(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::Num(n) => *n,
        Expr::Add(a, b) => eval(a) + eval(b),
        Expr::Mul(a, b) => eval(a) * eval(b),
    }
}

// 2. Trait objects.
// Different types have different sizes, so they can't be stored side by side in a Vec.
// `Box<dyn Shape>` stores each one on the heap and keeps a same-sized pointer in the Vec.
trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> &'static str;
}

struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    fn name(&self) -> &'static str {
        "circle"
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
    fn name(&self) -> &'static str {
        "rectangle"
    }
}

fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

// ---------- Arc<T> ----------
// Arc = "Atomically Reference Counted". It lets several owners share one heap value.
// Cloning an Arc does not copy the data, it only increments a counter.
// The value is dropped when the last Arc pointing to it is dropped.
// The counter is updated atomically, so an Arc can be sent to other threads
// (its single-threaded sibling, Rc<T>, can't).

// 3. Sharing read-only data between threads.
fn sum_in_threads(numbers: Arc<Vec<i32>>, thread_count: usize) -> i32 {
    let chunk_size = numbers.len().div_ceil(thread_count);

    let handles: Vec<_> = (0..thread_count)
        .map(|i| {
            // Each thread gets its own Arc pointing to the same Vec.
            let numbers = Arc::clone(&numbers);
            thread::spawn(move || {
                let start = (i * chunk_size).min(numbers.len());
                let end = (start + chunk_size).min(numbers.len());
                numbers[start..end].iter().sum::<i32>()
            })
        })
        .collect();

    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

// 4. Sharing mutable data between threads.
// Arc only gives shared (read-only) access. To change the value, wrap it in a Mutex,
// which makes sure only one thread at a time can modify it.
fn count_in_threads(thread_count: usize, increments_per_thread: usize) -> usize {
    let counter = Arc::new(Mutex::new(0));

    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..increments_per_thread {
                    *counter.lock().unwrap() += 1;
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    *counter.lock().unwrap()
}

fn main() {
    println!("--- Box: recursive type ---");
    // (2 + 3) * 4
    let expr = Expr::Mul(
        Box::new(Expr::Add(Box::new(Expr::Num(2)), Box::new(Expr::Num(3)))),
        Box::new(Expr::Num(4)),
    );
    println!("{:?} = {}", expr, eval(&expr));

    println!("\n--- Box: trait objects ---");
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Rectangle { width: 2.0, height: 3.0 }),
    ];
    for shape in &shapes {
        println!("{} has area {:.2}", shape.name(), shape.area());
    }
    println!("total area: {:.2}", total_area(&shapes));

    println!("\n--- Arc: reference counting ---");
    let data = Arc::new(String::from("shared"));
    println!("strong count after creating: {}", Arc::strong_count(&data));
    {
        let _second = Arc::clone(&data);
        println!("strong count after cloning: {}", Arc::strong_count(&data));
    } // `_second` is dropped here
    println!("strong count after the clone is dropped: {}", Arc::strong_count(&data));

    println!("\n--- Arc: read-only data shared between threads ---");
    let numbers = Arc::new((1..=100).collect::<Vec<i32>>());
    println!("sum of 1..=100 using 4 threads: {}", sum_in_threads(numbers, 4));

    println!("\n--- Arc<Mutex<T>>: mutable data shared between threads ---");
    println!("8 threads x 1000 increments: {}", count_in_threads(8, 1000));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_recursive_expression() {
        // 1 + (2 * 3)
        let expr = Expr::Add(
            Box::new(Expr::Num(1)),
            Box::new(Expr::Mul(Box::new(Expr::Num(2)), Box::new(Expr::Num(3)))),
        );
        assert_eq!(eval(&expr), 7);
    }

    #[test]
    fn test_trait_objects_in_vec() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Rectangle { width: 2.0, height: 3.0 }),
            Box::new(Rectangle { width: 1.0, height: 4.0 }),
        ];
        assert_eq!(total_area(&shapes), 10.0);
    }

    #[test]
    fn test_arc_strong_count() {
        let data = Arc::new(5);
        let clone = Arc::clone(&data);
        assert_eq!(Arc::strong_count(&data), 2);
        drop(clone);
        assert_eq!(Arc::strong_count(&data), 1);
    }

    #[test]
    fn test_sum_in_threads() {
        let numbers = Arc::new((1..=100).collect::<Vec<i32>>());
        assert_eq!(sum_in_threads(Arc::clone(&numbers), 3), 5050);
        // More threads than items still works: the extra threads get empty slices.
        assert_eq!(sum_in_threads(Arc::new(vec![1, 2]), 5), 3);
    }

    #[test]
    fn test_count_in_threads() {
        assert_eq!(count_in_threads(8, 1000), 8000);
    }
}
