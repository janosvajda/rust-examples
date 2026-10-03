<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Proxy Pattern

## What is it?

The **Proxy** pattern puts a stand-in in front of an object. The stand-in has exactly the same interface as the real thing, so callers can't tell the difference, but it can do something extra before or after passing the request on.

Think of a company's assistant who handles calls for the boss. Callers ask their questions as if they were talking to the boss. The assistant answers the ones they already know ("the office closes at 6"), and only puts the rest through. The boss gets far fewer interruptions, and the callers don't need to change how they ask.

## The parts

```text
                               ┌──────────────────────┐
  your code ──► ExchangeRates  │ CachingProxy         │  already know the answer?
               (the interface) │  - cache             │ ──── yes ──► answer from the cache
                               │  - service ──────────┼──── no ───► RemoteRateService (slow)
                               └──────────────────────┘             then remember its answer
```

- **Subject** (`ExchangeRates` trait): the interface both the real object and the proxy implement.
- **Real subject** (`RemoteRateService`): does the actual work. Here it's slow, like a network call.
- **Proxy** (`CachingProxy`): holds the real subject and decides when to actually call it.

## The example

`RemoteRateService` returns currency exchange rates, but every call takes 200 ms because it "goes over the network". `CachingProxy` wraps it and remembers every answer.

The demo makes six conversion requests. Only three are new questions (EUR→HUF, EUR→USD, GBP→JPY), so the slow service is only called three times. Repeated questions come back instantly. The timings printed next to each line show the difference.

Two details worth noticing:
- **"No answer" is cached too.** Asking for an unknown currency pair twice doesn't hit the service twice.
- **`convert` doesn't know it's using a proxy.** It accepts anything that implements `ExchangeRates`, and the tests pass both the real service and the proxy to it.

## Kinds of proxy

Caching is one use. The structure is the same for all of these; only what the proxy does differs:

| Kind | What the proxy does | Example |
|---|---|---|
| **Caching proxy** | remembers results of expensive calls | this example |
| **Virtual proxy** | delays creating an expensive object until it's really needed | loading a large image only when it's shown |
| **Protection proxy** | checks permissions before passing a request on | only admins may call `delete()` |
| **Remote proxy** | makes an object on another machine look local | calling a web service through a client library |
| **Logging proxy** | records every call | auditing, debugging |

## The Rust way

- The proxy is **generic** (`CachingProxy<S: ExchangeRates>`), so it can wrap any implementation, including a fast fake in the tests.
- `rate` takes `&self`, because to the caller asking for a rate is a read. The proxy still needs to update its cache, so it keeps the cache in a `RefCell`, which allows changes through a shared reference.
- Rust's own smart pointers are proxies too: `Rc`, `Arc` and `Box` stand in for a value and add something (shared ownership, heap allocation), while letting you use the value as if you held it directly.

**Proxy or Decorator?** They're built the same way: a wrapper with the same interface. The difference is the intent. A decorator *adds features* that callers ask for, like milk in a coffee. A proxy *controls access* to the real object, and callers usually don't know it's there.

## When to use it

- The real object is slow, expensive or remote, and some calls can be avoided or delayed.
- Access to the real object needs checks, limits or logging.
- You can't or don't want to change the real object, but need to control how it's used.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p proxy-pattern`.
