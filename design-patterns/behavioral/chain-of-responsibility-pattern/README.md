<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# Chain of Responsibility Pattern

## What is it?

The **Chain of Responsibility** pattern passes a request along a line of handlers. Each handler either deals with the request or passes it on to the next one. The sender doesn't need to know who will end up handling it.

Think of calling customer support. First an automated menu tries to help. If it can't, you're passed to a support agent. If the agent can't solve it, they escalate to a specialist, then maybe to a manager. You only ever made one call. Each person decides "can I handle this, or do I pass it up?"

## The parts

```text
  submit(expense)
        │
        ▼
  Policy check ───── invalid? ─────────────► rejected
        │ pass on
        ▼
  Team lead ──────── up to 500 €? ─────────► approved
        │ pass on
        ▼
  Department head ── up to 5,000 €? ───────► approved
        │ pass on
        ▼
  Director ───────── up to 50,000 €? ──────► approved
        │ pass on
        ▼
  nobody left ─────────────────────────────► rejected
```

- **Handler** (`Approver` trait): one method, `decide`, which returns a decision or `None` meaning "not mine, pass it on".
- **Concrete handlers** (`PolicyCheck`, `Manager`): each decides based on its own rules.
- **The chain** (`ApprovalChain`): holds the handlers in order and walks a request along them until one decides.

## The example

A company approves expenses based on the amount:

| Expense | Amount | Who decides |
|---|---:|---|
| Team lunch | 180 € | team lead approves (limit 500 €) |
| New laptop | 1,900 € | team lead passes it on → department head approves |
| Conference trip | 24,000 € | passed twice → director approves |
| New office building | 2,000,000 € | over everyone's limit → falls off the end, rejected |
| Casino night | 300 € | policy check rejects it before any manager sees it |
| Missing amount | 0 € | policy check rejects it |

Notice the policy check at the front: a link doesn't have to *approve* things. It can also stop bad requests early, so nobody further along wastes time on them.

**Adding a new rule doesn't change any existing code.** Need a CFO above the director, or a fraud check after the policy check? Add a link with `.then(...)` in the right place.

## The Rust way

- **The chain is a `Vec<Box<dyn Approver>>`.** The classic version has each handler hold a pointer to the next one, like a linked list. A `Vec` is simpler in Rust and easy to reorder.
- **`find_map` walks the chain in one line.** It calls `decide` on each link in order and stops at the first one that returns `Some`. That's exactly "pass it on until someone handles it".
- **`Option` says "not mine" clearly:** `None` means pass it on, `Some(decision)` means handled.
- **Builder-style setup:** `.then(link)` returns the chain, so building it reads top to bottom in the order requests travel.

You'll meet this pattern in web servers: **middleware** in frameworks like `axum` or `actix-web` is a chain of handlers (logging, authentication, rate limiting) that each look at a request and either respond or pass it on.

## When to use it

- Several handlers might deal with a request, and which one should is decided at runtime.
- You want to add, remove or reorder handling steps without changing the code that sends requests.
- Typical examples: approval workflows, support escalation, web middleware, event handling in user interfaces (a click goes to the button, then its panel, then the window).

**Watch out for:** requests that fall off the end unhandled. Decide what should happen then. Here it's an explicit rejection.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p chain-of-responsibility-pattern`.
