<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# A small HTTP server

Ferris runs a tiny café. Ask for exactly `GET /?hello`, and the café replies `world`. A different GET target receives `404`; another method receives `405` with `Allow: GET`.

```bash
cargo run
# In another terminal:
curl -i 'http://127.0.0.1:3000/?hello'
curl -i 'http://127.0.0.1:3000/?other'
curl -i -X POST 'http://127.0.0.1:3000/?hello'
```

Run commands from this directory. `cargo test` checks fragmented requests, exact routing, incomplete headers and the size limit.

## Why reading once is insufficient

TCP carries an ordered stream of bytes. A request can arrive in several pieces: first `G`, then `ET /?hello HTTP/1.1\r\n`, then the remaining headers. One `read` does not mean one complete request. The server reads through the header terminator `\r\n\r\n` before interpreting the request line.

Routing uses the method and target from that line. Writing `GET /?hello` inside another header cannot change the route. Responses include their byte `Content-Length`, a content type, and `Connection: close`.

## Resource limits

| Limit | Value | Purpose |
|---|---|---|
| worker threads | 8 | bounds concurrent handlers |
| queued connections | 16 | bounds accepted connections waiting for a worker |
| request header bytes | 8,192, including the terminator | prevents unlimited header allocation |
| socket read/write timeout | 5 seconds per operation | bounds a stalled socket operation |

A full queue causes the newly accepted connection to close. Malformed or incomplete headers receive `400` when a response can still be written; a read timeout receives `408`. Clients sending a slow trickle can keep individual reads progressing, so the socket timeout is **not an overall request deadline**.

## Scope

This is a standard-library implementation of a small HTTP subset: HTTP/1.0 or HTTP/1.1 request lines, CRLF-delimited headers, one request per connection, and the routes above. It does not implement full header validation, request bodies, keep-alive, TLS, chunked transfer encoding or HTTP/2. For a general HTTP service, use an established HTTP implementation and configure its limits.

HTTP began as the protocol for exchanging documents on the Web. Here the document is just five bytes: `world`!
