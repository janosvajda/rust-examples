// Chain of Responsibility pattern: expense approval. A request is passed
// along a chain of approvers. Each one either handles it (approves or
// rejects) or passes it to the next person up. The employee submitting the
// expense only talks to the first link in the chain.

#[derive(Debug)]
struct Expense {
    description: String,
    amount_eur: u32,
}

impl Expense {
    fn new(description: &str, amount_eur: u32) -> Expense {
        Expense {
            description: description.to_string(),
            amount_eur,
        }
    }
}

/// The final outcome of a request.
#[derive(Debug, PartialEq)]
enum Decision {
    Approved { by: String },
    Rejected { by: String, reason: String },
}

/// One link in the chain.
trait Approver {
    /// Either decides, or returns `None` to say "not mine, pass it on".
    fn decide(&self, expense: &Expense) -> Option<Decision>;
    fn name(&self) -> String;
}

// ---- The links ---------------------------------------------------------------

/// Approves anything up to their spending limit.
struct Manager {
    title: &'static str,
    limit_eur: u32,
}

impl Approver for Manager {
    fn decide(&self, expense: &Expense) -> Option<Decision> {
        if expense.amount_eur <= self.limit_eur {
            Some(Decision::Approved { by: self.name() })
        } else {
            None // over my limit: someone above me must decide
        }
    }

    fn name(&self) -> String {
        format!("{} (limit {} €)", self.title, self.limit_eur)
    }
}

/// Not every link approves things. This one sits at the front of the chain
/// and rejects obviously invalid requests before anyone wastes time on them.
struct PolicyCheck;

impl Approver for PolicyCheck {
    fn decide(&self, expense: &Expense) -> Option<Decision> {
        if expense.amount_eur == 0 {
            return Some(Decision::Rejected {
                by: self.name(),
                reason: "amount is zero".to_string(),
            });
        }
        if expense.description.to_lowercase().contains("casino") {
            return Some(Decision::Rejected {
                by: self.name(),
                reason: "not a business expense".to_string(),
            });
        }
        None // looks fine, pass it on
    }

    fn name(&self) -> String {
        "policy check".to_string()
    }
}

// ---- The chain ---------------------------------------------------------------

/// Holds the links in order and walks the request along them.
struct ApprovalChain {
    links: Vec<Box<dyn Approver>>,
}

impl ApprovalChain {
    fn new() -> Self {
        ApprovalChain { links: Vec::new() }
    }

    /// Adds a link to the end of the chain. Returns `self` so calls can be
    /// chained: `ApprovalChain::new().then(a).then(b)`.
    fn then(mut self, link: impl Approver + 'static) -> Self {
        self.links.push(Box::new(link));
        self
    }

    /// Passes the expense along until a link decides. If nobody does, the
    /// request falls off the end of the chain, so we reject it.
    fn submit(&self, expense: &Expense) -> Decision {
        self.links
            .iter()
            .find_map(|link| link.decide(expense))
            .unwrap_or_else(|| Decision::Rejected {
                by: "nobody".to_string(),
                reason: "over every approver's limit".to_string(),
            })
    }
}

fn main() {
    // Build the chain once. Order matters: cheapest decision-makers first.
    let chain = ApprovalChain::new()
        .then(PolicyCheck)
        .then(Manager { title: "Team lead", limit_eur: 500 })
        .then(Manager { title: "Department head", limit_eur: 5_000 })
        .then(Manager { title: "Director", limit_eur: 50_000 });

    let expenses = [
        Expense::new("Team lunch", 180),
        Expense::new("New laptop", 1_900),
        Expense::new("Conference trip for 10 people", 24_000),
        Expense::new("New office building", 2_000_000),
        Expense::new("Casino night", 300),
        Expense::new("Forgot to fill in the amount", 0),
    ];

    for expense in &expenses {
        let outcome = match chain.submit(expense) {
            Decision::Approved { by } => format!("approved by {by}"),
            Decision::Rejected { by, reason } => format!("REJECTED by {by}: {reason}"),
        };
        println!("{:<32} {:>9} €  →  {outcome}", expense.description, expense.amount_eur);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chain() -> ApprovalChain {
        ApprovalChain::new()
            .then(PolicyCheck)
            .then(Manager { title: "Lead", limit_eur: 100 })
            .then(Manager { title: "Boss", limit_eur: 1_000 })
    }

    fn approved_by(decision: Decision) -> String {
        match decision {
            Decision::Approved { by } => by,
            other => panic!("expected approval, got {other:?}"),
        }
    }

    #[test]
    fn the_first_link_that_can_decide_does() {
        assert_eq!(approved_by(chain().submit(&Expense::new("pens", 20))), "Lead (limit 100 €)");
        assert_eq!(approved_by(chain().submit(&Expense::new("chair", 500))), "Boss (limit 1000 €)");
    }

    #[test]
    fn limits_are_inclusive() {
        assert_eq!(approved_by(chain().submit(&Expense::new("x", 100))), "Lead (limit 100 €)");
        assert_eq!(approved_by(chain().submit(&Expense::new("x", 101))), "Boss (limit 1000 €)");
    }

    #[test]
    fn falling_off_the_end_is_a_rejection() {
        let decision = chain().submit(&Expense::new("yacht", 1_000_000));
        assert!(matches!(decision, Decision::Rejected { by, .. } if by == "nobody"));
    }

    #[test]
    fn policy_check_stops_bad_requests_early() {
        let decision = chain().submit(&Expense::new("Casino trip", 50));
        assert!(matches!(decision, Decision::Rejected { by, .. } if by == "policy check"));
    }

    #[test]
    fn an_empty_chain_rejects_everything() {
        let decision = ApprovalChain::new().submit(&Expense::new("anything", 1));
        assert!(matches!(decision, Decision::Rejected { .. }));
    }
}
