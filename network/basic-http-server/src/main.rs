use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

const WORKER_COUNT: usize = 8;
const MAX_QUEUED_CONNECTIONS: usize = 16;
const MAX_HEADER_BYTES: usize = 8 * 1024;
const IO_TIMEOUT: Duration = Duration::from_secs(5);

// Read complete headers with a size limit. TCP may split a request anywhere.
fn read_request(reader: &mut impl BufRead) -> io::Result<(String, String)> {
    let mut limited = reader.take(MAX_HEADER_BYTES as u64 + 1);
    let mut headers = Vec::new();
    loop {
        if limited.read_until(b'\n', &mut headers)? == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "incomplete headers",
            ));
        }
        if headers.len() > MAX_HEADER_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "headers too large",
            ));
        }
        if headers.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let text =
        std::str::from_utf8(&headers).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let line = text.split("\r\n").next().unwrap_or("");
    let parts: Vec<_> = line.split(' ').collect();
    match parts.as_slice() {
        [method, target, "HTTP/1.0" | "HTTP/1.1"] if target.starts_with('/') => {
            Ok((method.to_string(), target.to_string()))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid request line",
        )),
    }
}

fn respond(stream: &mut impl Write, status: &str, body: &str) -> io::Result<()> {
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n", body.len())?;
    if status.starts_with("405") {
        write!(stream, "Allow: GET\r\n")?;
    }
    write!(stream, "\r\n{body}")
}

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    let request = read_request(&mut BufReader::new(&stream));
    let (status, body) = match request {
        Ok((method, _)) if method != "GET" => ("405 Method Not Allowed", "Only GET is supported"),
        Ok((_, target)) if target == "/?hello" => ("200 OK", "world"),
        Ok(_) => ("404 Not Found", "404 Not Found"),
        Err(e)
            if matches!(
                e.kind(),
                io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
            ) =>
        {
            ("408 Request Timeout", "Request timed out")
        }
        Err(_) => ("400 Bad Request", "Invalid or incomplete request"),
    };
    respond(&mut stream, status, body)
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000")?;
    let (sender, receiver) = mpsc::sync_channel::<TcpStream>(MAX_QUEUED_CONNECTIONS);
    let receiver = Arc::new(Mutex::new(receiver));
    thread::scope(|scope| {
        for _ in 0..WORKER_COUNT {
            let receiver = Arc::clone(&receiver);
            scope.spawn(move || loop {
                // Release the queue lock before processing a connection.
                let next = receiver.lock().expect("queue lock is not poisoned").recv();
                let Ok(stream) = next else { break };
                if let Err(error) = handle_client(stream) {
                    eprintln!("connection failed: {error}");
                }
            });
        }
        for incoming in listener.incoming() {
            let stream = match incoming {
                Ok(stream) => stream,
                Err(error) => {
                    eprintln!("accept failed: {error}");
                    continue;
                }
            };
            match sender.try_send(stream) {
                Ok(()) => {}
                // Closing the connection keeps overload handling bounded too.
                Err(mpsc::TrySendError::Full(_)) => eprintln!("connection queue is full"),
                Err(mpsc::TrySendError::Disconnected(_)) => break,
            }
        }
        drop(sender);
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn exchange(chunks: &[&[u8]]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let worker = thread::spawn(move || handle_client(listener.accept().unwrap().0));
        let mut stream = TcpStream::connect(address).unwrap();
        stream.set_read_timeout(Some(IO_TIMEOUT)).unwrap();
        for chunk in chunks {
            stream.write_all(chunk).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        worker.join().unwrap().unwrap();
        response
    }

    #[test]
    fn fragmented_request_waits_for_complete_headers() {
        let response = exchange(&[b"G", b"ET /?hello HTTP/1.1\r\n", b"Host: localhost\r\n\r\n"]);
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("Content-Length: 5\r\n"));
        assert!(response.ends_with("\r\n\r\nworld"));
    }

    #[test]
    fn routes_only_the_request_target() {
        let response = exchange(&[b"GET /other HTTP/1.1\r\nX-Test: GET /?hello\r\n\r\n"]);
        assert!(response.starts_with("HTTP/1.1 404"));
        assert!(exchange(&[b"POST /?hello HTTP/1.1\r\n\r\n"]).starts_with("HTTP/1.1 405"));
    }

    #[test]
    fn incomplete_or_oversized_headers_are_rejected() {
        assert!(read_request(&mut Cursor::new(b"GET / HTTP/1.1\r\n")).is_err());
        assert!(read_request(&mut Cursor::new(vec![b'x'; MAX_HEADER_BYTES + 1])).is_err());
    }
}
