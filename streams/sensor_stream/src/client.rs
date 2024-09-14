use std::net::TcpStream;
use std::io::{self, BufReader, BufRead};
use std::time::Duration;
use std::thread;

// Simulates processing each chunk of data received from the server
fn process_stream(stream: TcpStream) -> io::Result<()> {
    let reader = BufReader::new(stream);

    // Process each line of data received from the server
    for line in reader.lines() {
        let line = line?;
        println!("Received: {}", line);

        // Simulate processing delay
        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}

fn main() -> io::Result<()> {
    // Connect to the server
    let stream = TcpStream::connect("127.0.0.1:8080")?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;

    println!("Connected to server, waiting for data...");

    // Process incoming data from the server
    if let Err(e) = process_stream(stream) {
        eprintln!("Error processing stream: {}", e);
    }

    Ok(())
}
