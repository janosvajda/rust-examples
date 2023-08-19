use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

// Function to handle each client connection
fn handle_client(mut stream: TcpStream) {
    // Create a buffer to store incoming data
    let mut buffer = [0; 1024];
    // Read data from the stream into the buffer
    stream.read(&mut buffer).unwrap();

    // Convert the buffer to a string
    let request = String::from_utf8_lossy(&buffer);
    // Define the response based on the request
    let response = if request.contains("GET /?hello") {
        "HTTP/1.1 200 OK\r\n\r\nworld" // Respond with "world" if request contains "hello"
    } else {
        "HTTP/1.1 404 NOT FOUND\r\n\r\n404 Not Found" // Respond with a 404 error otherwise
    };

    // Write the response to the stream
    stream.write(response.as_bytes()).unwrap();
    // Flush the stream to ensure the response is sent
    stream.flush().unwrap();
}

fn main() {
    // Bind the server to address 127.0.0.1:3000
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();

    // Loop to accept incoming client connections
    for stream in listener.incoming() {
        // Unwrap the stream from the Result
        let stream = stream.unwrap();

        // Spawn a new thread to handle the client communication
        thread::spawn(|| {
            handle_client(stream);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    // Test for handling a client connection with "hello" parameter
    #[test]
    fn test_handle_client_hello() {
        // Bind the listener to an available port
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        // Get the bound address
        let addr = listener.local_addr().unwrap();
        // Spawn a thread to simulate a client connection
        let handle = std::thread::spawn(move || {
            let stream = TcpStream::connect(addr).unwrap();
            handle_client(stream);
        });

        // Accept the connection from the spawned client thread
        let (mut stream, _) = listener.accept().unwrap();
        // Send a test GET request with the "hello" parameter
        stream.write(b"GET /?hello HTTP/1.1\r\n\r\n").unwrap();

        // Read the response from the server
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        // Join the spawned thread
        handle.join().unwrap();

        // Check if the response contains the expected status and message
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("world"));
    }

    // Test for handling a client connection without "hello" parameter
    #[test]
    fn test_handle_client_not_found() {
        // Bind the listener to an available port
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        // Get the bound address
        let addr = listener.local_addr().unwrap();
        // Spawn a thread to simulate a client connection
        let handle = std::thread::spawn(move || {
            let stream = TcpStream::connect(addr).unwrap();
            handle_client(stream);
        });

        // Accept the connection from the spawned client thread
        let (mut stream, _) = listener.accept().unwrap();
        // Send a test GET request without the "hello" parameter
        stream.write(b"GET /?other HTTP/1.1\r\n\r\n").unwrap();

        // Read the response from the server
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        // Join the spawned thread
        handle.join().unwrap();

        // Check if the response contains the expected status and message
        assert!(response.contains("HTTP/1.1 404 NOT FOUND"));
        assert!(response.contains("404 Not Found"));
    }
}