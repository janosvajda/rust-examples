// A normal (std) program using the no_std library. A no_std library works
// in std programs too: `core` is part of `std`. That's what makes no_std
// crates so reusable: one library, every platform.

use core::fmt::Write;
use no_std_core_library::{
    ByteRing, FixedVec, PACKET_LEN, PacketError, SensorReading, StackString, crc32,
};

fn main() {
    println!("1. FixedVec: a Vec with a capacity fixed at compile time");
    let mut readings: FixedVec<i16, 3> = FixedVec::new();
    for value in [215, 198, 203, 190] {
        match readings.push(value) {
            Ok(()) => println!("    stored {value}"),
            Err(rejected) => println!(
                "    full (capacity {}): {rejected} was given back",
                readings.capacity()
            ),
        }
    }

    println!("\n2. ByteRing: bytes arriving from a 'serial port'");
    let mut uart: ByteRing<16> = ByteRing::new();
    let sent = SensorReading {
        sensor_id: 7,
        temperature_tenths: 215,
        humidity_percent: 48,
    }
    .to_bytes();
    let mut dropped = 0;
    for &byte in sent.iter().chain(sent.iter()) {
        if !uart.write(byte) {
            dropped += 1;
        }
    }
    println!(
        "    18 bytes arrived, {} buffered, {dropped} dropped (buffer full)",
        uart.len()
    );

    println!("\n3. CRC-32, table computed at compile time");
    println!(
        "    crc32(\"123456789\") = {:#010X} (the standard check value is 0xCBF43926)",
        crc32(b"123456789")
    );

    println!("\n4. Parsing binary packets");
    let mut packet = [0u8; PACKET_LEN];
    for slot in packet.iter_mut() {
        *slot = uart.read().unwrap_or(0);
    }
    match SensorReading::parse(&packet) {
        Ok(reading) => println!("    {packet:02X?}\n    → {reading}"),
        Err(e) => println!("    error: {e:?}"),
    }
    let mut corrupted = packet;
    corrupted[2] ^= 0x80; // radio noise flips a bit
    match SensorReading::parse(&corrupted) {
        Ok(reading) => println!("    {reading}"),
        Err(PacketError::ChecksumMismatch { expected, actual }) => {
            println!("    corrupted packet rejected: checksum {actual:#010X} ≠ {expected:#010X}")
        }
        Err(e) => println!("    error: {e:?}"),
    }

    println!("\n5. Formatting without a heap");
    let mut line: StackString<48> = StackString::new();
    let reading = SensorReading {
        sensor_id: 2,
        temperature_tenths: -15,
        humidity_percent: 91,
    };
    write!(line, "{reading}").unwrap();
    println!(
        "    \"{}\" ({} bytes, all on the stack)",
        line.as_str(),
        line.as_str().len()
    );

    println!("\n6. Memory: everything has a size known at compile time");
    println!(
        "    FixedVec<i16, 3>: {} bytes",
        size_of::<FixedVec<i16, 3>>()
    );
    println!("    ByteRing<16>:     {} bytes", size_of::<ByteRing<16>>());
    println!(
        "    StackString<48>:  {} bytes",
        size_of::<StackString<48>>()
    );
}
