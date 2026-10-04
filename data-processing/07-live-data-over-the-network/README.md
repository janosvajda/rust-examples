<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 7: Live data over the network

## The idea in one sentence

Over a network, everything that can go wrong eventually will: servers stall, connections drop, data arrives garbled, twice or not at all, and strangers try to connect. A real client and server treat each of these as a **normal event** with a planned answer.

## Client and server: what, why, and where it came from

### What it means

Two programs, two roles:
- The **server** waits. It owns something: data, a service, a device. It answers whoever asks.
- The **client** starts the conversation. It asks for something, then uses the answer.

The words describe **roles**, not computers. One machine can run both, as `cargo run` does here. A program can also be a server to some programs and a client of others: a web server is a client of its database.

### A short history

| When | What happened |
|---|---|
| **1960s** | **Time-sharing.** One expensive central computer, many simple terminals. The terminals only showed text and sent keystrokes; all the work happened in the centre. The idea of "many users, one central machine" starts here. |
| **1969** | **ARPANET**, the network that grew into the Internet, connects its first computers. Its first technical note, RFC 1, is about the software that lets one "host" computer talk to another. |
| **1970s** | At **Xerox PARC**, networked personal workstations share a **file server** and a **print server** over the newly invented **Ethernet** (1973). Each desk does its own work, and the network shares what's expensive. That's client–server in its modern form. |
| **1983** | The ARPANET switches to **TCP/IP** on 1 January, the protocols this lesson uses. The same year, BSD Unix introduces the **sockets** interface, which is still how programs open network connections. Rust's `TcpListener` and `TcpStream` are a safe wrapper around it. |
| **1980s** | Personal computers in offices, linked by local networks, talk to shared **file servers** and **database servers**. "Client–server computing" becomes the standard way to build business software. |
| **1989–1991** | Tim Berners-Lee at CERN invents the **World Wide Web**: a **browser** (the client) asks a **web server** for pages over **HTTP**. It's the most successful client–server system ever built. |
| **today** | Phone apps talking to APIs, websites talking to databases, sensors sending data to the cloud, services talking to other services. Almost every program that uses a network is a client, a server, or both. |

### Why it's useful

| Benefit | Example |
|---|---|
| **One source of truth** | every client sees the same data, kept in one place: a bank balance can't differ between your phone and your laptop |
| **Sharing what's expensive** | one powerful database, GPU or sensor array serves many cheap clients |
| **Control and security** | the server decides who may do what. This lesson's server checks the token (authentication) and the permitted sensors (authorization) before sending a single reading. A client can't skip a check that runs on someone else's machine. |
| **Update in one place** | fix a bug in the server, and every client benefits at once, with nothing to reinstall |
| **Many kinds of client** | a phone app, a website and a command-line tool can all use the same server |
| **Scaling** | when there are more clients, add more servers, without changing the clients |

### The downsides, and the alternatives

- **The server is a single point of failure.** If it's down, every client is stuck. That's why this lesson's client retries with backoff, and why real systems run several servers.
- **Everything depends on the network.** Slow, broken or missing connections are normal, which is what most of this lesson handles.
- **The server must handle everyone's load.** That's why this server limits its clients (`ERR busy`).

Other designs exist for other needs:
- **Peer-to-peer:** every program is client and server at once, with no centre. BitTorrent shares files this way.
- **Local-first:** each device keeps its own full copy, and syncs when it can. Some note-taking apps work this way.
- **No network at all:** many programs just don't need one.

### How this lesson fits in

The server here owns the sensor data, and checks who's asking. The client asks, then processes the readings as they arrive. Between them there's a **protocol**: the agreed format of every message (`HELLO …`, `OK`, `ERR …`, the reading lines). Every client–server system has one. The Web's is HTTP. This lesson's is a few lines of text that you can read, and type yourself with a tool like `nc 127.0.0.1 8080`.

Here the **client pulls**: it connects and asks. Many sensor systems work the other way round: each sensor is a client that **pushes** its readings to a central server. The problems are the same in both directions: timeouts, retries, duplicates, losses and authentication.

## Three programs

| Command | What it does |
|---|---|
| `cargo run` | the full demo: a server that misbehaves on purpose in every way below, a client that copes, then a wrong token, a forbidden sensor, and no server at all |
| `cargo run --bin server` | a server on `127.0.0.1:8080`, one reading per second; add `-- faults` to make it misbehave |
| `SENSOR_TOKEN=demo-token cargo run --bin client` | a client for that server, in a second terminal; add `-- B22` for another sensor |

The code is in a small library, `src/lib.rs`, with three modules:

| Module | Contains |
|---|---|
| `protocol.rs` | the message format, validation, and a line reader that can't be made to use unbounded memory |
| `server.rs` | authentication, authorization, connection limits, timeouts, and the deliberate faults |
| `client.rs` | timeouts, retries with backoff, resuming, duplicate and loss detection, the bounded state |

## The protocol

```text
client → server   HELLO <token> <sensor> <first-seq>      who I am, what I want, where to start
server → client   OK                                       or: ERR unauthenticated / forbidden / busy / bad-request
server → client   <seq>,<time>,<sensor>,<value>,<unit>     one line per reading
                  27,12:00:27,A12,21.36,C
```

Every reading carries a **sequence number** (`seq`): 0, 1, 2, … That single number is what makes reconnecting safe. The client asks to resume from a number, recognises a number it already has (a duplicate) and notices a number that never came (a loss).

## The demo: everything goes wrong, nothing is lost

```text
connected, resuming from seq 0
#0   12:00:00   22.24 °C   moving average 22.24
     rejected: malformed line
     rejected: not valid UTF-8
#8   12:00:08   37.81 °C   ⚠ ALERT: above 30
     rejected: line too long
     rejected: value is NaN or infinite
     duplicate #16 ignored
disconnected: the server closed the connection
retry 1 in 45.35ms
connected, resuming from seq 22
     duplicate #20 ignored
     duplicate #21 ignored
disconnected: no data for 500ms: the server seems stalled
retry 1 in 49.47ms
connected, resuming from seq 27
     ⚠ reading #33 was lost
     rejected: value outside -50..=150 °C

accepted 37, alerts 4, duplicates ignored 5, lost 1, reconnects 2
rejected: {TooLong: 1, NotUtf8: 1, Malformed: 1, NotFinite: 1, OutOfRange: 1}
every reading accounted for: next expected #40
```

37 accepted + 2 rejected (with their sequence numbers) + 1 lost = all 40 readings, each counted exactly once, through two reconnects and five duplicates. A test checks exactly that.

## Edge cases

### The server or the network

| Edge case | What happens | Test |
|---|---|---|
| **server not running** | `Connection refused` → retry with exponential backoff (doubling, with a cap), then give up with a clear error | `an_unavailable_server_is_retried_then_given_up` |
| **host unreachable** (packets silently dropped) | `connect_timeout` gives up after 500 ms instead of the operating system's minutes | (same code path) |
| **server stalls** (connected, but sends nothing) | `set_read_timeout`: after 500 ms of silence → reconnect and resume | `every_fault_is_survived_and_accounted_for` |
| **connection drops mid-stream** | the read sees the end → reconnect, resume from `next_seq` | same |
| **server busy** (too many clients) | `ERR busy` → retried with backoff, because it may be free later | `too_many_clients_are_turned_away` |
| **many clients retry at once** after an outage | backoff has **jitter**: a random 50–100% of the delay, so they don't all hit the server at the same instant | `backoff_doubles_up_to_the_cap` |
| **it worked for a while, then failed** | the retry counter resets after progress, so a long-running client isn't killed by its fifth hiccup in a month | (in `run`) |

### Messy data

| Edge case | What happens | Test |
|---|---|---|
| **garbage** (`%%% garbage %%%`) | rejected: malformed | `every_kind_of_bad_line_is_named` |
| **invalid UTF-8** bytes | rejected, and the stream continues. `BufRead::lines()` would return an error here, and a `line?` loop would stop at the first bad byte | `every_fault_is_survived_and_accounted_for` |
| **a gigantic line** (100 KB, or 10 MB, without a newline) | rejected after 256 bytes; the rest is skipped in chunks, never stored | `a_gigantic_line_never_fills_memory` |
| **`NaN`, `inf`, `1e400`** | rejected. `"NaN".parse::<f64>()` **succeeds** in Rust, and `1e400` becomes infinity, so they must be checked explicitly | `every_kind_of_bad_line_is_named` |
| **an impossible value** (500 °C) | rejected: outside -50..=150 °C | same |
| **wrong sensor or unit** | rejected: we only process what we asked for | same |
| **Windows line endings** (`\r\n`) | accepted: the `\r` is removed | `lines_are_split_and_line_endings_removed` |

A rejected line whose sequence number could still be read counts as **arrived but unusable**, not as lost. So every number is accounted for exactly once.

### Duplicates, losses and idempotency

| Edge case | What happens | Test |
|---|---|---|
| **the same reading sent twice** | recognised by its `seq` and ignored | `duplicates_change_nothing` |
| **resent after a reconnect** | the server re-sends the last 2 readings (**at-least-once** delivery, as many real systems do); the client ignores them | `every_fault_is_survived_and_accounted_for` |
| **a reading never arrives** | the next `seq` jumps: reported as lost (`⚠ reading #33 was lost`) | `a_skipped_sequence_number_is_reported_as_lost` |

Processing is **idempotent**: applying the same reading a second time changes nothing. The test applies a reading twice and checks that the state is identical. Combined with at-least-once delivery, the result is **effectively exactly-once**: every reading counted once, even though the network delivered some twice. This is the standard way real systems get there, because "exactly once" delivery on its own isn't possible over an unreliable network.

### Who may connect: authentication and authorization

| Edge case | What happens | Test |
|---|---|---|
| **unknown token** (authentication: *who are you?*) | `ERR unauthenticated` → the client stops and **doesn't retry**: retrying a wrong password is pointless, and many servers would lock the account | `a_wrong_token_is_refused_without_retrying` |
| **valid token, forbidden sensor** (authorization: *what may you do?*) | `ERR forbidden` → no retry. `guest-token` may read B22, but not A12 | `a_valid_token_cannot_read_another_sensor` |
| **a malformed request** | `ERR bad-request` → no retry: it's a bug, and repeating it won't help | (in `try_session`) |
| **a client that connects and says nothing** | the server's read timeout closes it after 2 s, so it can't hold a thread forever | `a_silent_client_is_disconnected_by_the_server` |
| **a client that stops reading** | the server's write timeout ends that stream after 2 s, freeing its slot | (in `handle_client`) |
| **the token in the source code** | it isn't: the client reads it from the `SENSOR_TOKEN` environment variable. Secrets in source files end up in version control | (in `src/bin/client.rs`) |

### Memory over a long run

| Edge case | What happens | Test |
|---|---|---|
| **running for months** | every part of the client's state has a fixed size: one `next_seq`, a 5-value window, one counter per kind of rejection. Nothing grows with the stream | `memory_stays_bounded_over_a_long_run` (1,000,000 readings) |
| **one line that never ends** | capped at 256 bytes (above) | `a_gigantic_line_never_fills_memory` |

Measured on a real run over TCP, with the release build on an Apple M2:

| Readings streamed | Peak memory of the whole program |
|---|---|
| 200,000 | 1.84 MB |
| 2,000,000 | 1.85 MB |

Ten times more data, the same memory. In Rust, "memory leak" rarely means forgotten `free` calls, because ownership frees memory automatically. The real risks are **collections that only grow**: a `Vec` of every reading, a `HashSet` of every seen id, a log kept in memory. The fix is to keep a summary and a bounded window, never the whole history. (Rust can truly leak through `Rc` reference cycles or `mem::forget`, and this code uses neither.)

## Not covered here, on purpose

These matter in production. Each needs either a crate or a lot of code that would bury the lesson:

| Topic | Why it matters | Where to look |
|---|---|---|
| **encryption (TLS)** | on plain TCP, anyone on the network can read the token and the data | `rustls`, or a TLS-terminating proxy |
| **constant-time token comparison** | comparing secrets with `==` or a `HashMap` lookup can leak timing information | the `subtle` crate, or let a well-tested auth library compare |
| **token expiry and rotation** | a leaked token should stop working | short-lived tokens (for example JWT or OAuth) |
| **remembering `next_seq` across restarts** | a restarted client starts at 0 again | save `next_seq` to a file after processing, then resume from it |
| **graceful shutdown** on Ctrl+C | finish the current reading, save state, then exit | the `ctrlc` crate, or `tokio::signal` in async code |
| **rate limiting** | one client shouldn't be able to use all the server's capacity | a token bucket per client |
| **many connections cheaply** | a thread per client costs memory; thousands of clients need async I/O | the [async course](../../async-await/) and tokio |

## Run it

```bash
cargo run                                          # the full demo
cargo test                                         # 15 tests, one per edge case

cargo run --bin server -- faults                   # terminal 1
SENSOR_TOKEN=demo-token cargo run --bin client     # terminal 2
SENSOR_TOKEN=guest-token cargo run --bin client    # terminal 3: forbidden
```

Previous: [Lesson 6: A processing pipeline](../06-processing-pipeline/) · Next: [Lesson 8: Joining datasets](../08-joining-datasets/)
