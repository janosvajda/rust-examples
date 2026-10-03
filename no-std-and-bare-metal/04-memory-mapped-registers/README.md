<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: Memory-mapped registers

## The idea in one sentence

On a microcontroller, hardware is controlled by reading and writing **special memory addresses called registers**, and Rust code that does this must use **volatile** access, exact memory layouts and careful bit manipulation.

## How software talks to hardware

A chip's peripherals (serial ports, timers, GPIO pins, ADCs) each appear as a small block of registers at fixed addresses, listed in the chip's datasheet:

```text
  address        register         meaning
  0x4000_1000    UART data        write a byte here → the hardware sends it
  0x4000_1004    UART status      bit 0 = "ready to send"   (set by the hardware)
  0x4000_1008    UART control     bit 0 = enable, bit 1 = transmit, bit 4 = parity
  0x4000_100C    UART baud        clock frequency ÷ baud rate
```

Reading or writing those addresses doesn't touch RAM. It talks to the hardware. This lesson writes a real-style serial port (UART) driver. The only difference from real firmware is that the "hardware" is simulated in ordinary memory, so the program runs on your computer. On a real chip, the registers are found like this:

```rust
const UART0: *const UartRegisters = 0x4000_1000 as *const UartRegisters;
let uart = unsafe { &*UART0 };
```

## Three rules for register code

**1. Every access must be volatile.** The compiler normally assumes memory only changes when your code changes it, so it may delete a "pointless" second read, or merge two writes. For registers that's wrong: the status register changes **on its own** as the hardware works, and every write to the data register **sends a byte**. `read_volatile` and `write_volatile` mean "do exactly this access, exactly here". The `Register` type wraps them, so driver code can't forget.

**2. The layout must match the datasheet exactly.**

```rust
#[repr(C)]
pub struct UartRegisters {
    pub data: Register,          // 0x0
    pub status: Register,        // 0x4
    pub control: Register,       // 0x8
    pub baud_divisor: Register,  // 0xC
}
```

Rust may normally reorder struct fields. `#[repr(C)]` forbids that, and `#[repr(transparent)]` makes each `Register` exactly one `u32`. The demo prints the offsets, and a test checks them with `offset_of!`.

**3. Change bits with read-modify-write.** One register usually holds several independent settings, one per bit. To change one bit without disturbing the others:

| Operation | Code |
|---|---|
| set a bit | `bits \| FLAG` |
| clear a bit | `bits & !FLAG` |
| test a bit | `bits & FLAG != 0` |
| a flag constant | `const CONTROL_PARITY: u32 = 1 << 4;` |

```text
after setting PARITY:  0b00010000
after setting ENABLE:  0b00010011
after clearing PARITY: 0b00000011
```

## A safe driver on top

The rest of the program never touches registers or bits. It uses the `Uart` driver:

```rust
let uart = Uart::enable(&registers, 115_200, false);   // configure, then enable
uart.try_send(b'H')    // Ok(()), or Err(WouldBlock) while the hardware is busy
```

`try_send` doesn't wait in a loop. If the hardware is still sending the previous byte, it returns `WouldBlock`, and the caller can do other useful work and try again. That's how real embedded drivers are often designed: the `embedded-hal` crate's traits work this way. In the demo, sending "Hello, hardware!" hit `WouldBlock` 15 times, once per byte after the first, while the simulated hardware caught up.

## In real projects

Nobody writes register structs by hand for a whole chip. Chip vendors publish a machine-readable description (an SVD file), and the `svd2rust` tool generates a **Peripheral Access Crate** with a type-safe API for every register and bit, like `stm32f4` or `rp2040-pac`. On top of that sit **HAL crates** with drivers like this lesson's `Uart`. The ideas, volatile access, exact layout and bit fields, are exactly the ones here.

## Run it

```bash
cargo run
cargo test
```

The register and driver code uses only `core`. The demo's simulated hardware uses `std` (a `Vec` to record what was "sent"), because it stands in for the physical world.

Previous: [Lesson 3: A custom allocator](../03-custom-allocator/) · Next: [Lesson 5: WebAssembly without an OS](../05-webassembly-without-an-os/)
