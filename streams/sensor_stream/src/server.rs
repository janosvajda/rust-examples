use std::net::{TcpListener, TcpStream};
use std::io::{Write, Result};
use std::thread;
use std::time::Duration;
use rand::Rng;

// Simulates sending sensor data to the client
fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut rng = rand::thread_rng();

    loop {
        // Simulate sensor data (temperature)
        let temperature: f64 = rng.gen_range(15.0..35.0);

        // Prepare data to send (as bytes)
        let data = format!("Temperature: {:.2} °C\n", temperature);

        // Write data to the stream
        stream.write_all(data.as_bytes())?;

        // Simulate delay between readings
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() -> Result<()> {
    // Bind to a local TCP address
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    println!("Server listening on port 8080...");

    // Accept connections and spawn a new thread for each client
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a thread to handle the connection
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}
