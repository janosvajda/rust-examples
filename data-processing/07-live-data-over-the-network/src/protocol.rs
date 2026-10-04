//! The protocol: what client and server say to each other, and how every
//! incoming line is checked before anyone trusts it.
//!
//! ```text
//! client → server   HELLO <token> <sensor> <first-seq>     (one line, then only listens)
//! server → client   OK                                      (or ERR <reason>, then closes)
//! server → client   <seq>,<time>,<sensor>,<value>,<unit>    (one line per reading, forever)
//! ```
//!
//! `seq` is a sequence number: 0, 1, 2, … for every reading. It's what makes
//! reconnecting safe: the client asks to resume from a number, notices
//! duplicates (a number it already has) and losses (a number that was skipped).

use std::fmt;
use std::io::{self, BufRead, Read};

/// The longest line anyone will accept. A peer that sends more without a
/// newline, by mistake or on purpose, can't make us use more memory.
pub const MAX_LINE_BYTES: usize = 256;

/// One validated reading.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub seq: u64,
    pub time: String,
    pub sensor: String,
    pub celsius: f64,
}

/// Why an incoming line was refused. Every variant is counted, never a crash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rejection {
    TooLong,
    NotUtf8,
    Malformed,
    NotANumber,
    NotFinite,
    OutOfRange,
    WrongSensor,
    WrongUnit,
}

impl fmt::Display for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Rejection::TooLong => "line too long",
            Rejection::NotUtf8 => "not valid UTF-8",
            Rejection::Malformed => "malformed line",
            Rejection::NotANumber => "value is not a number",
            Rejection::NotFinite => "value is NaN or infinite",
            Rejection::OutOfRange => "value outside -50..=150 °C",
            Rejection::WrongSensor => "reading for a sensor we didn't ask for",
            Rejection::WrongUnit => "unit is not C",
        })
    }
}

/// A refused line. If its sequence number could still be read, it's kept:
/// the reading DID arrive, it was just unusable, so it isn't "lost".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bad {
    pub seq: Option<u64>,
    pub why: Rejection,
}

/// Checks one line completely. Only a line that passes every check becomes a `Message`.
pub fn parse_message(line: &str, expected_sensor: &str) -> Result<Message, Bad> {
    let fields: Vec<&str> = line.split(',').collect();
    let [seq, time, sensor, value, unit] = fields[..] else {
        return Err(Bad { seq: None, why: Rejection::Malformed });
    };
    let seq: u64 = seq.parse().map_err(|_| Bad { seq: None, why: Rejection::Malformed })?;
    let bad = |why| Bad { seq: Some(seq), why };
    if sensor != expected_sensor {
        return Err(bad(Rejection::WrongSensor));
    }
    if unit != "C" {
        return Err(bad(Rejection::WrongUnit));
    }
    // `"NaN".parse::<f64>()` and `"inf".parse()` succeed, so check explicitly.
    let celsius: f64 = value.parse().map_err(|_| bad(Rejection::NotANumber))?;
    if !celsius.is_finite() {
        return Err(bad(Rejection::NotFinite));
    }
    if !(-50.0..=150.0).contains(&celsius) {
        return Err(bad(Rejection::OutOfRange));
    }
    Ok(Message { seq, time: time.to_string(), sensor: sensor.to_string(), celsius })
}

/// What `read_line_limited` found.
#[derive(Debug, PartialEq)]
pub enum LineRead {
    /// A complete line, without its newline, in the buffer.
    Line,
    /// A line longer than the limit. It was skipped up to its newline,
    /// without ever being held in memory.
    TooLong,
    /// The other side closed the connection.
    Closed,
}

/// Reads one line of at most `max` bytes into `buf` (which is cleared first).
///
/// `BufRead::read_line` would grow its buffer for as long as the peer keeps
/// sending without a newline, so a single gigantic line could use up all our
/// memory. This reads at most `max + 1` bytes, and discards anything beyond.
pub fn read_line_limited(input: &mut impl BufRead, buf: &mut Vec<u8>, max: usize) -> io::Result<LineRead> {
    buf.clear();
    let n = Read::take(&mut *input, max as u64 + 1).read_until(b'\n', buf)?;
    if n == 0 {
        return Ok(LineRead::Closed);
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
        if buf.last() == Some(&b'\r') {
            buf.pop(); // accept Windows line endings too
        }
        return Ok(LineRead::Line);
    }
    if buf.len() <= max {
        return Ok(LineRead::Line); // the last line, without a final newline
    }
    // Too long: throw the rest of the line away, chunk by chunk.
    buf.clear();
    loop {
        let available = input.fill_buf()?;
        if available.is_empty() {
            return Ok(LineRead::TooLong);
        }
        match available.iter().position(|&b| b == b'\n') {
            Some(i) => {
                input.consume(i + 1);
                return Ok(LineRead::TooLong);
            }
            None => {
                let len = available.len();
                input.consume(len);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_message() {
        let m = parse_message("7,12:00:07,A12,21.5,C", "A12").unwrap();
        assert_eq!((m.seq, m.celsius), (7, 21.5));
    }

    #[test]
    fn every_kind_of_bad_line_is_named() {
        let check = |line| parse_message(line, "A12").unwrap_err().why;
        assert_eq!(check("%%% garbage %%%"), Rejection::Malformed);
        assert_eq!(check("x,t,A12,21.5,C"), Rejection::Malformed); // seq isn't a number
        assert_eq!(check("1,t,A12,warm,C"), Rejection::NotANumber);
        assert_eq!(check("1,t,A12,NaN,C"), Rejection::NotFinite);
        assert_eq!(check("1,t,A12,inf,C"), Rejection::NotFinite);
        assert_eq!(check("1,t,A12,1e400,C"), Rejection::NotFinite); // too big: becomes infinity
        assert_eq!(check("1,t,A12,500,C"), Rejection::OutOfRange);
        assert_eq!(check("1,t,B22,21.5,C"), Rejection::WrongSensor);
        assert_eq!(check("1,t,A12,70.7,F"), Rejection::WrongUnit);
        // the sequence number survives when only the value is bad
        assert_eq!(parse_message("9,t,A12,NaN,C", "A12").unwrap_err().seq, Some(9));
        assert_eq!(parse_message("garbage", "A12").unwrap_err().seq, None);
    }

    #[test]
    fn lines_are_split_and_line_endings_removed() {
        let mut input: &[u8] = b"first\r\nsecond\nlast without newline";
        let mut buf = Vec::new();
        for expected in ["first", "second", "last without newline"] {
            assert_eq!(read_line_limited(&mut input, &mut buf, 64).unwrap(), LineRead::Line);
            assert_eq!(buf, expected.as_bytes());
        }
        assert_eq!(read_line_limited(&mut input, &mut buf, 64).unwrap(), LineRead::Closed);
    }

    #[test]
    fn a_gigantic_line_never_fills_memory() {
        // 10 MB without a newline, then a normal line.
        let mut data = vec![b'x'; 10_000_000];
        data.extend_from_slice(b"\nnext\n");
        let mut input = &data[..];
        let mut buf = Vec::new();
        assert_eq!(read_line_limited(&mut input, &mut buf, 256).unwrap(), LineRead::TooLong);
        assert!(buf.capacity() < 4096, "the buffer stayed small: {} bytes", buf.capacity());
        assert_eq!(read_line_limited(&mut input, &mut buf, 256).unwrap(), LineRead::Line);
        assert_eq!(buf, b"next"); // and reading continues normally afterwards
    }
}
