//! A `no_std` library: everything here uses only `core`, so it works on a
//! microcontroller with no operating system and no heap, in a browser, or in
//! an ordinary program. No `Vec`, no `String`, no `Box`: all memory is either
//! on the stack or fixed in size at compile time.
//!
//! `cargo test` builds with `std` (the test harness needs it), which is why
//! the attribute below only applies outside tests.
#![cfg_attr(not(test), no_std)]

use core::fmt;

// ============================================================================
// 1. A fixed-capacity Vec: const generics instead of a heap
// ============================================================================

/// Like `Vec<T>`, but the capacity `N` is part of the TYPE and the items live
/// inline (on the stack, or inside whatever contains the FixedVec). It never
/// allocates, so pushing into a full one returns the item back as an error
/// instead of growing.
///
/// Real projects use the `heapless` crate for this; it stores items in
/// `MaybeUninit` to avoid the `Option` overhead. `Option` keeps this version
/// 100% safe code.
pub struct FixedVec<T, const N: usize> {
    items: [Option<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    pub fn new() -> Self {
        FixedVec {
            items: core::array::from_fn(|_| None), // works for any T, no Copy needed
            len: 0,
        }
    }

    /// Adds an item, or gives it back if there's no room left.
    pub fn push(&mut self, item: T) -> Result<(), T> {
        if self.len == N {
            return Err(item);
        }
        self.items[self.len] = Some(item);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.items[self.len].take()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items[..self.len].iter().filter_map(Option::as_ref)
    }
}

impl<T, const N: usize> Default for FixedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 2. A ring buffer for incoming bytes (like a UART receive buffer)
// ============================================================================

/// Bytes arrive one at a time (from a serial port, a radio…) and are read
/// later. A fixed array used as a circle: no allocation, constant time.
pub struct ByteRing<const N: usize> {
    buffer: [u8; N],
    head: usize, // next byte to read
    len: usize,
}

impl<const N: usize> ByteRing<N> {
    pub const fn new() -> Self {
        ByteRing {
            buffer: [0; N],
            head: 0,
            len: 0,
        }
    }

    /// Stores a byte. Returns false (and drops the byte) if the buffer is
    /// full: on a microcontroller you can't wait for space, data keeps coming.
    pub fn write(&mut self, byte: u8) -> bool {
        if self.len == N {
            return false;
        }
        self.buffer[(self.head + self.len) % N] = byte;
        self.len += 1;
        true
    }

    pub fn read(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }
        let byte = self.buffer[self.head];
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(byte)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const N: usize> Default for ByteRing<N> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 3. CRC-32 with a table computed at COMPILE time
// ============================================================================

/// The 256-entry CRC-32 lookup table, built by a `const fn` while compiling.
/// At runtime it's just 1 KB of constant data in the binary (in flash memory
/// on a microcontroller), with zero startup cost.
const CRC32_TABLE: [u32; 256] = build_crc32_table();

const fn build_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        // `for` loops aren't allowed in const fn yet; `while` is.
        let mut crc = i as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
            bit += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

/// The standard CRC-32 (as used by ZIP, PNG and Ethernet) of `data`.
/// Detects corrupted bytes in a message.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF;
    for &byte in data {
        crc = CRC32_TABLE[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    !crc
}

// ============================================================================
// 4. Parsing a binary packet from bytes, without allocating
// ============================================================================

/// A reading sent by a sensor as 9 bytes:
///
/// ```text
///   byte  0      1        2-3                4          5-8
///       ┌──────┬────────┬──────────────────┬──────────┬───────────────────┐
///       │ 0xAA │ sensor │ temperature × 10 │ humidity │ CRC-32 of bytes   │
///       │ sync │ id     │ i16, big-endian  │ %        │ 0-4, big-endian   │
///       └──────┴────────┴──────────────────┴──────────┴───────────────────┘
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SensorReading {
    pub sensor_id: u8,
    pub temperature_tenths: i16,
    pub humidity_percent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketError {
    WrongLength(usize),
    BadSyncByte(u8),
    ChecksumMismatch { expected: u32, actual: u32 },
}

pub const PACKET_LEN: usize = 9;
const SYNC: u8 = 0xAA;

impl SensorReading {
    /// Builds the 9 bytes for this reading, into an array: no allocation.
    pub fn to_bytes(&self) -> [u8; PACKET_LEN] {
        let mut bytes = [0u8; PACKET_LEN];
        bytes[0] = SYNC;
        bytes[1] = self.sensor_id;
        bytes[2..4].copy_from_slice(&self.temperature_tenths.to_be_bytes());
        bytes[4] = self.humidity_percent;
        let crc = crc32(&bytes[..5]);
        bytes[5..9].copy_from_slice(&crc.to_be_bytes());
        bytes
    }

    /// Checks and decodes 9 bytes. Every way the bytes can be wrong is a
    /// variant of PacketError, not a panic: firmware must not crash on noise.
    pub fn parse(bytes: &[u8]) -> Result<SensorReading, PacketError> {
        let bytes: &[u8; PACKET_LEN] = bytes
            .try_into()
            .map_err(|_| PacketError::WrongLength(bytes.len()))?;
        if bytes[0] != SYNC {
            return Err(PacketError::BadSyncByte(bytes[0]));
        }
        let expected = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
        let actual = crc32(&bytes[..5]);
        if expected != actual {
            return Err(PacketError::ChecksumMismatch { expected, actual });
        }
        Ok(SensorReading {
            sensor_id: bytes[1],
            temperature_tenths: i16::from_be_bytes([bytes[2], bytes[3]]),
            humidity_percent: bytes[4],
        })
    }
}

// ============================================================================
// 5. Formatting text without a heap: core::fmt::Write into a stack buffer
// ============================================================================

/// A string with a fixed maximum length, stored inline. Implementing
/// `core::fmt::Write` makes `write!` work on it, which is how `no_std`
/// code formats numbers into text for a display or a serial port.
pub struct StackString<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> StackString<N> {
    pub const fn new() -> Self {
        StackString {
            bytes: [0; N],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        // Only whole &str values are ever copied in, so this is valid UTF-8.
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> Default for StackString<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> fmt::Write for StackString<N> {
    /// Appends the text, or fails (without writing a partial character) if
    /// it doesn't fit.
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        if end > N {
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

impl fmt::Display for SensorReading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.temperature_tenths / 10;
        let tenth = (self.temperature_tenths % 10).abs();
        let sign = if self.temperature_tenths < 0 && whole == 0 {
            "-"
        } else {
            ""
        };
        write!(
            f,
            "sensor {}: {sign}{whole}.{tenth} °C, {}% humidity",
            self.sensor_id, self.humidity_percent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;

    #[test]
    fn fixed_vec_refuses_to_grow() {
        let mut v: FixedVec<&str, 2> = FixedVec::new();
        assert_eq!(v.push("a"), Ok(()));
        assert_eq!(v.push("b"), Ok(()));
        assert_eq!(v.push("c"), Err("c")); // full: the item comes back
        assert_eq!(v.iter().copied().collect::<Vec<_>>(), ["a", "b"]);
        assert_eq!(v.pop(), Some("b"));
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn ring_buffer_wraps_around() {
        let mut ring: ByteRing<3> = ByteRing::new();
        assert!(ring.write(1) && ring.write(2) && ring.write(3));
        assert!(!ring.write(4)); // full
        assert_eq!(ring.read(), Some(1));
        assert!(ring.write(4)); // reuses the freed slot
        assert_eq!(
            [ring.read(), ring.read(), ring.read(), ring.read()],
            [Some(2), Some(3), Some(4), None]
        );
    }

    #[test]
    fn crc32_matches_the_standard_check_value() {
        // The official check value of CRC-32 for the ASCII text "123456789".
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn packet_round_trip() {
        let reading = SensorReading {
            sensor_id: 7,
            temperature_tenths: -45,
            humidity_percent: 63,
        };
        let bytes = reading.to_bytes();
        assert_eq!(SensorReading::parse(&bytes), Ok(reading));
    }

    #[test]
    fn corrupted_packets_are_rejected() {
        let mut bytes = SensorReading {
            sensor_id: 1,
            temperature_tenths: 215,
            humidity_percent: 40,
        }
        .to_bytes();
        assert_eq!(
            SensorReading::parse(&bytes[..5]),
            Err(PacketError::WrongLength(5))
        );
        bytes[3] ^= 0x01; // flip one bit of the temperature
        assert!(matches!(
            SensorReading::parse(&bytes),
            Err(PacketError::ChecksumMismatch { .. })
        ));
        bytes[0] = 0x00;
        assert_eq!(
            SensorReading::parse(&bytes),
            Err(PacketError::BadSyncByte(0))
        );
    }

    #[test]
    fn formatting_into_a_stack_buffer() {
        let mut text: StackString<64> = StackString::new();
        let reading = SensorReading {
            sensor_id: 3,
            temperature_tenths: -5,
            humidity_percent: 80,
        };
        write!(text, "{reading}").unwrap();
        assert_eq!(text.as_str(), "sensor 3: -0.5 °C, 80% humidity");

        let mut tiny: StackString<4> = StackString::new();
        assert!(write!(tiny, "too long").is_err()); // doesn't fit: an error, not a crash
    }
}
