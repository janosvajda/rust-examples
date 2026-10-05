// State pattern: an online shop order that behaves differently depending on
// where it is in its lifecycle. You can cancel a pending order but not a
// delivered one; you can ship a paid order but not an unpaid one.

use std::fmt;

/// Every state an order can be in. Some states carry extra data that only
/// makes sense in that state, like a tracking number once it's shipped.
#[derive(Debug, Clone, PartialEq)]
enum OrderState {
    Pending,
    Paid { amount: u32 },
    Shipped { tracking: String },
    Delivered,
    Cancelled { reason: String },
}

/// What went wrong when an action isn't allowed in the current state.
#[derive(Debug, PartialEq)]
struct InvalidAction {
    action: &'static str,
    state: OrderState,
}

impl fmt::Display for InvalidAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot {} an order that is {:?}",
            self.action, self.state
        )
    }
}

/// The order. Its behaviour depends entirely on `state`.
struct Order {
    id: u32,
    state: OrderState,
}

impl Order {
    fn new(id: u32) -> Self {
        Order {
            id,
            state: OrderState::Pending,
        }
    }

    /// Each action looks at the current state and either moves to the next
    /// state or refuses. All the rules live in one place: these `match`es.
    ///
    /// A `Pending` order can be paid.
    fn pay(&mut self, amount: u32) -> Result<(), InvalidAction> {
        match self.state {
            OrderState::Pending => {
                self.state = OrderState::Paid { amount };
                Ok(())
            }
            _ => self.refuse("pay"),
        }
    }

    /// A `Paid` order can be shipped.
    fn ship(&mut self, tracking: &str) -> Result<(), InvalidAction> {
        match self.state {
            OrderState::Paid { .. } => {
                self.state = OrderState::Shipped {
                    tracking: tracking.to_string(),
                };
                Ok(())
            }
            _ => self.refuse("ship"),
        }
    }

    /// A `Shipped` order can be marked as delivered.
    fn deliver(&mut self) -> Result<(), InvalidAction> {
        match self.state {
            OrderState::Shipped { .. } => {
                self.state = OrderState::Delivered;
                Ok(())
            }
            _ => self.refuse("deliver"),
        }
    }

    /// An order can be cancelled until it leaves the warehouse.
    fn cancel(&mut self, reason: &str) -> Result<(), InvalidAction> {
        match self.state {
            OrderState::Pending | OrderState::Paid { .. } => {
                if let OrderState::Paid { amount } = self.state {
                    println!("  refunding {amount} € for order #{}", self.id);
                }
                self.state = OrderState::Cancelled {
                    reason: reason.to_string(),
                };
                Ok(())
            }
            _ => self.refuse("cancel"),
        }
    }

    /// Behaviour that isn't a transition can depend on the state too.
    fn status_message(&self) -> String {
        match &self.state {
            OrderState::Pending => "waiting for payment".to_string(),
            OrderState::Paid { amount } => format!("paid {amount} €, preparing to ship"),
            OrderState::Shipped { tracking } => format!("on its way, tracking number {tracking}"),
            OrderState::Delivered => "delivered, enjoy!".to_string(),
            OrderState::Cancelled { reason } => format!("cancelled ({reason})"),
        }
    }

    fn refuse(&self, action: &'static str) -> Result<(), InvalidAction> {
        Err(InvalidAction {
            action,
            state: self.state.clone(),
        })
    }
}

/// Prints the result of an action and the order's new status.
fn report(order: &Order, action: &str, result: Result<(), InvalidAction>) {
    match result {
        Ok(()) => println!(
            "{action:<8} ok      → #{} is {}",
            order.id,
            order.status_message()
        ),
        Err(error) => println!("{action:<8} refused → {error}"),
    }
}

fn main() {
    println!("--- The happy path ---");
    let mut order = Order::new(1);
    println!("#1 is {}", order.status_message());
    let result = order.pay(49);
    report(&order, "pay", result);
    let result = order.ship("HU-123456");
    report(&order, "ship", result);
    let result = order.deliver();
    report(&order, "deliver", result);

    println!("\n--- Actions that aren't allowed ---");
    let result = order.cancel("changed my mind");
    report(&order, "cancel", result); // too late: already delivered

    let mut order = Order::new(2);
    let result = order.ship("HU-999");
    report(&order, "ship", result); // not paid yet

    println!("\n--- Cancelling a paid order ---");
    let result = order.pay(120);
    report(&order, "pay", result);
    let result = order.cancel("found it cheaper elsewhere");
    report(&order, "cancel", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_the_normal_lifecycle() {
        let mut order = Order::new(1);
        assert_eq!(order.state, OrderState::Pending);
        order.pay(10).unwrap();
        assert_eq!(order.state, OrderState::Paid { amount: 10 });
        order.ship("T1").unwrap();
        assert_eq!(
            order.state,
            OrderState::Shipped {
                tracking: "T1".to_string()
            }
        );
        order.deliver().unwrap();
        assert_eq!(order.state, OrderState::Delivered);
    }

    #[test]
    fn cannot_skip_steps() {
        let mut order = Order::new(1);
        assert!(order.ship("T1").is_err());
        assert!(order.deliver().is_err());
        // A refused action leaves the state unchanged.
        assert_eq!(order.state, OrderState::Pending);
    }

    #[test]
    fn cannot_pay_twice() {
        let mut order = Order::new(1);
        order.pay(10).unwrap();
        let error = order.pay(10).unwrap_err();
        assert_eq!(error.action, "pay");
        assert_eq!(error.state, OrderState::Paid { amount: 10 });
    }

    #[test]
    fn can_cancel_only_before_shipping() {
        let mut pending = Order::new(1);
        assert!(pending.cancel("test").is_ok());

        let mut paid = Order::new(2);
        paid.pay(10).unwrap();
        assert!(paid.cancel("test").is_ok());

        let mut shipped = Order::new(3);
        shipped.pay(10).unwrap();
        shipped.ship("T3").unwrap();
        assert!(shipped.cancel("test").is_err());
    }

    #[test]
    fn cancelled_orders_are_final() {
        let mut order = Order::new(1);
        order.cancel("test").unwrap();
        assert!(order.pay(10).is_err());
        assert!(order.cancel("again").is_err());
        assert_eq!(order.status_message(), "cancelled (test)");
    }
}
