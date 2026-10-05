// Lesson 4: memory-mapped registers.
//
// On a microcontroller, hardware is controlled through REGISTERS: special
// memory addresses where reading or writing talks to a peripheral (a serial
// port, a timer, a GPIO pin) instead of RAM. For example:
//
//     address 0x4000_1000   UART data register     write a byte → it's sent
//     address 0x4000_1004   UART status register   bit 0 = "ready to send"
//
// This lesson writes a real-style UART (serial port) driver. The only
// difference from real firmware: the "hardware" is simulated in ordinary
// memory, so the program runs anywhere.

use core::cell::UnsafeCell;
use core::ptr::{read_volatile, write_volatile};

// ============================================================================
// 1. One register: volatile access only
// ============================================================================

/// A 32-bit hardware register.
///
/// Every access is VOLATILE. Normally the compiler may remove or merge
/// memory accesses it thinks are pointless: reading the same address twice
/// in a row, or writing a value that's overwritten right after. For a
/// register that's wrong: a status register changes on its own as the
/// hardware works, and every write to a data register sends a byte.
/// `read_volatile` / `write_volatile` tell the compiler "do exactly this
/// access, exactly here".
#[repr(transparent)] // same layout as a plain u32
pub struct Register(UnsafeCell<u32>);

impl Register {
    pub const fn new(value: u32) -> Self {
        Register(UnsafeCell::new(value))
    }

    pub fn read(&self) -> u32 {
        // SAFETY: the pointer comes from a live UnsafeCell<u32>.
        unsafe { read_volatile(self.0.get()) }
    }

    pub fn write(&self, value: u32) {
        // SAFETY: as above.
        unsafe { write_volatile(self.0.get(), value) }
    }

    /// Read-modify-write: change some bits, keep the others.
    pub fn modify(&self, change: impl FnOnce(u32) -> u32) {
        self.write(change(self.read()));
    }
}

// ============================================================================
// 2. A peripheral: a block of registers with a fixed layout
// ============================================================================

/// The UART's registers, in the order the chip's datasheet gives them.
/// `#[repr(C)]` guarantees the fields are laid out in exactly this order,
/// 4 bytes apart, with no reordering: they must match the hardware.
#[repr(C)]
pub struct UartRegisters {
    pub data: Register,         // offset 0x0: write a byte to send it
    pub status: Register,       // offset 0x4: flags set by the hardware
    pub control: Register,      // offset 0x8: flags set by the software
    pub baud_divisor: Register, // offset 0xC: clock ÷ baud rate
}

// Bits of the control register.
pub const CONTROL_ENABLE: u32 = 1 << 0;
pub const CONTROL_TX_ENABLE: u32 = 1 << 1;
pub const CONTROL_PARITY: u32 = 1 << 4;

// Bits of the status register.
pub const STATUS_TX_READY: u32 = 1 << 0;

/// On real hardware, the registers are found by casting a fixed address:
///
/// ```text
/// const UART0: *const UartRegisters = 0x4000_1000 as *const UartRegisters;
/// let uart = unsafe { &*UART0 };
/// ```
///
/// Here they're an ordinary value in memory, standing in for the chip.
pub fn fake_uart() -> UartRegisters {
    UartRegisters {
        data: Register::new(0),
        status: Register::new(STATUS_TX_READY),
        control: Register::new(0),
        baud_divisor: Register::new(0),
    }
}

// ============================================================================
// 3. A safe driver on top of the raw registers
// ============================================================================

#[derive(Debug, PartialEq, Eq)]
pub enum UartError {
    InvalidBaudRate,
    NotEnabled,
    /// The hardware is still busy sending the previous byte: try again later.
    WouldBlock,
}

/// The rest of the program uses this, and never touches registers or bits.
pub struct Uart<'a> {
    registers: &'a UartRegisters,
}

impl<'a> Uart<'a> {
    pub const CLOCK_HZ: u32 = 16_000_000;

    /// Configures the UART: baud rate, parity, then switches it on. The
    /// order matters on real chips: configure first, enable last.
    pub fn enable(
        registers: &'a UartRegisters,
        baud: u32,
        parity: bool,
    ) -> Result<Self, UartError> {
        let divisor = Self::CLOCK_HZ
            .checked_div(baud)
            .filter(|&n| n > 0)
            .ok_or(UartError::InvalidBaudRate)?;
        registers.baud_divisor.write(divisor);
        registers.control.modify(|bits| {
            let bits = if parity {
                bits | CONTROL_PARITY
            } else {
                bits & !CONTROL_PARITY
            };
            bits | CONTROL_ENABLE | CONTROL_TX_ENABLE
        });
        Ok(Uart { registers })
    }

    /// Tries to send one byte without waiting. Embedded drivers often work
    /// this way, so the program can do other things while the hardware is
    /// busy, instead of spinning in a loop.
    pub fn try_send(&self, byte: u8) -> Result<(), UartError> {
        if self.registers.control.read() & (CONTROL_ENABLE | CONTROL_TX_ENABLE)
            != (CONTROL_ENABLE | CONTROL_TX_ENABLE)
        {
            return Err(UartError::NotEnabled);
        }
        if self.registers.status.read() & STATUS_TX_READY == 0 {
            return Err(UartError::WouldBlock);
        }
        self.registers.data.write(byte as u32);
        // Writing data makes the hardware busy until it has sent the byte.
        self.registers.status.modify(|bits| bits & !STATUS_TX_READY);
        Ok(())
    }

    pub fn disable(&self) {
        self.registers
            .control
            .modify(|bits| bits & !(CONTROL_ENABLE | CONTROL_TX_ENABLE));
    }
}

// ============================================================================
// 4. The simulated hardware
// ============================================================================

/// Plays the role of the silicon: each "tick", a busy UART finishes sending
/// the byte in its data register and becomes ready again.
pub struct SimulatedWire {
    pub sent: Vec<u8>,
}

impl SimulatedWire {
    pub fn tick(&mut self, registers: &UartRegisters) {
        let busy = registers.status.read() & STATUS_TX_READY == 0;
        if busy {
            self.sent.push(registers.data.read() as u8);
            registers.status.modify(|bits| bits | STATUS_TX_READY);
        }
    }
}

/// Sends a whole message, letting the "hardware" run whenever it's busy.
pub fn send_message(uart: &Uart, wire: &mut SimulatedWire, text: &str) -> Result<usize, UartError> {
    let mut waits = 0;
    for &byte in text.as_bytes() {
        loop {
            match uart.try_send(byte) {
                Ok(()) => break,
                Err(UartError::WouldBlock) => {
                    wire.tick(uart.registers);
                    waits += 1;
                }
                Err(error) => return Err(error),
            }
        }
    }
    wire.tick(uart.registers);
    Ok(waits)
}

fn main() {
    let registers = fake_uart();

    println!("1. Bit manipulation on the control register");
    registers.control.write(0);
    registers.control.modify(|b| b | CONTROL_PARITY); // set a bit
    println!(
        "    after setting PARITY:  {:#010b}",
        registers.control.read()
    );
    registers
        .control
        .modify(|b| b | CONTROL_ENABLE | CONTROL_TX_ENABLE); // set two more
    println!(
        "    after setting ENABLE:  {:#010b}",
        registers.control.read()
    );
    registers.control.modify(|b| b & !CONTROL_PARITY); // clear a bit
    println!(
        "    after clearing PARITY: {:#010b}",
        registers.control.read()
    );
    println!(
        "    is TX enabled? {}",
        registers.control.read() & CONTROL_TX_ENABLE != 0
    );
    registers.control.write(0);

    println!("\n2. The register block's layout matches the datasheet");
    let base = &registers as *const UartRegisters as usize;
    for (name, field) in [
        ("data", &registers.data),
        ("status", &registers.status),
        ("control", &registers.control),
        ("baud_divisor", &registers.baud_divisor),
    ] {
        println!(
            "    {name:<13} at offset {:#04x}",
            field as *const Register as usize - base
        );
    }

    println!("\n3. A safe driver");
    let unconfigured = Uart {
        registers: &registers,
    };
    println!(
        "    sending before enabling: {:?}",
        unconfigured.try_send(b'x')
    );
    let uart = Uart::enable(&registers, 115_200, false).unwrap();
    println!(
        "    enabled at 115200 baud: divisor = {} (16 MHz ÷ 115200)",
        registers.baud_divisor.read()
    );

    let mut wire = SimulatedWire { sent: Vec::new() };
    let waits = send_message(&uart, &mut wire, "Hello, hardware!").unwrap();
    println!(
        "    the wire received {:?}, after waiting for the hardware {waits} times",
        String::from_utf8_lossy(&wire.sent)
    );
    uart.disable();
    println!("    disabled: control = {:#010b}", registers.control.read());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_have_the_datasheet_layout() {
        assert_eq!(size_of::<Register>(), 4);
        assert_eq!(size_of::<UartRegisters>(), 16);
        assert_eq!(core::mem::offset_of!(UartRegisters, status), 0x4);
        assert_eq!(core::mem::offset_of!(UartRegisters, baud_divisor), 0xC);
    }

    #[test]
    fn modify_keeps_the_other_bits() {
        let reg = Register::new(0b1010);
        reg.modify(|b| b | 0b0001);
        assert_eq!(reg.read(), 0b1011);
        reg.modify(|b| b & !0b1000);
        assert_eq!(reg.read(), 0b0011);
    }

    #[test]
    fn enable_sets_divisor_and_flags() {
        let regs = fake_uart();
        Uart::enable(&regs, 9_600, true).unwrap();
        assert_eq!(regs.baud_divisor.read(), 16_000_000 / 9_600);
        let control = regs.control.read();
        assert_eq!(
            control & (CONTROL_ENABLE | CONTROL_TX_ENABLE | CONTROL_PARITY),
            0b10011
        );
    }

    #[test]
    fn sending_waits_for_the_hardware() {
        let regs = fake_uart();
        let uart = Uart::enable(&regs, 115_200, false).unwrap();
        assert_eq!(uart.try_send(b'a'), Ok(()));
        assert_eq!(uart.try_send(b'b'), Err(UartError::WouldBlock)); // still busy
        let mut wire = SimulatedWire { sent: Vec::new() };
        wire.tick(&regs);
        assert_eq!(uart.try_send(b'b'), Ok(()));
        wire.tick(&regs);
        assert_eq!(wire.sent, b"ab");
    }

    #[test]
    fn whole_message_arrives() {
        let regs = fake_uart();
        let uart = Uart::enable(&regs, 115_200, false).unwrap();
        let mut wire = SimulatedWire { sent: Vec::new() };
        send_message(&uart, &mut wire, "OK").unwrap();
        assert_eq!(wire.sent, b"OK");
    }

    #[test]
    fn disabled_uart_refuses_to_send() {
        let regs = fake_uart();
        let uart = Uart::enable(&regs, 115_200, false).unwrap();
        uart.disable();
        assert_eq!(uart.try_send(b'x'), Err(UartError::NotEnabled));
    }
    #[test]
    fn invalid_configuration_and_disabled_transmission_are_errors() {
        let registers = fake_uart();
        assert!(matches!(
            Uart::enable(&registers, 0, false),
            Err(UartError::InvalidBaudRate)
        ));
        assert!(matches!(
            Uart::enable(&registers, Uart::CLOCK_HZ + 1, false),
            Err(UartError::InvalidBaudRate)
        ));
        let uart = Uart::enable(&registers, 9600, false).unwrap();
        registers.control.modify(|bits| bits & !CONTROL_TX_ENABLE);
        assert_eq!(uart.try_send(b'x'), Err(UartError::NotEnabled));
        let mut wire = SimulatedWire { sent: Vec::new() };
        assert_eq!(
            send_message(&uart, &mut wire, "x"),
            Err(UartError::NotEnabled)
        );
        assert!(wire.sent.is_empty());
    }
}
