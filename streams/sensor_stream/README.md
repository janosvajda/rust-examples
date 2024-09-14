# Rust TCP Stream Processing Example

## Overview

This Rust project demonstrates stream processing using TCP sockets. It consists of:

1. **Server**: Simulates a data source by streaming random temperature readings over TCP.
2. **Client**: Connects to the server, reads, and processes the stream in real-time.

---

## Project Structure

### Server (`src/server.rs`)

- **Function**: The server generates random temperature readings between 15°C and 35°C and sends them over TCP to connected clients.
- **Data Stream**: The temperature data is sent every second to simulate real-time data.

### Client (`src/client.rs`)

- **Function**: The client connects to the server, receives the temperature data, and processes it by printing it to the console.

---

## Prerequisites

- **Rust**: Ensure Rust is installed on your system. Install it by following the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).
- **Cargo**: Rust’s package manager (included with Rust) will be used to build and run the project.

---

## How to Run the Project

### Step 1: Clone the Repository

Clone the project to your local machine using Git:

```bash
git https://github.com/janosvajda/rust-examples
cd /rust-examples/streams/sensor_stream
```


### Step 2: Run server and client on a separated terminal

Clone the project to your local machine using Git:

```bash
cargo run --bin server

cargo run --bin client
```


Multiple clients: You could connect multiple clients to the server, each receiving the same data stream.