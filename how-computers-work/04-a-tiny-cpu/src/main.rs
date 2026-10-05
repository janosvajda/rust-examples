// Lesson 4: a tiny CPU.
//
// An invented 8-bit computer, small enough to understand completely:
//   memory     256 bytes, holding the program AND its data
//   registers  R0–R3: four bytes of very fast storage inside the CPU
//   PC         the program counter: the address of the next instruction
//
// Our model repeats three steps until HALT, an error or the step limit:
//   1. FETCH    read the instruction's bytes at the address in PC
//   2. DECODE   work out which instruction they are
//   3. EXECUTE  do it, and move PC to the next instruction

use std::fmt;

/// The instruction set. Each instruction is encoded as 1 to 3 bytes:
/// an opcode byte, followed by its operands.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Instruction {
    Halt,                                // 00            stop
    Load { reg: u8, value: u8 },         // 01 r v       R[r] = v
    Add { reg: u8, other: u8 },          // 02 r s       R[r] = R[r] + R[s]
    Sub { reg: u8, other: u8 },          // 03 r s       R[r] = R[r] - R[s]
    Read { reg: u8, addr: u8 },          // 04 r a       R[r] = memory[a]
    Write { reg: u8, addr: u8 },         // 05 r a       memory[a] = R[r]
    Jump { addr: u8 },                   // 06 a         PC = a
    JumpIfNotZero { reg: u8, addr: u8 }, // 07 r a   if R[r] != 0 { PC = a }
    Print { reg: u8 },                   // 08 r         show R[r]
}

use Instruction::*;

impl Instruction {
    const fn encoded_len(self) -> u8 {
        match self {
            Halt => 1,
            Jump { .. } | Print { .. } => 2,
            _ => 3,
        }
    }

    /// The instruction as machine code: the bytes the CPU actually reads.
    fn encode(self) -> Vec<u8> {
        match self {
            Halt => vec![0x00],
            Load { reg, value } => vec![0x01, reg, value],
            Add { reg, other } => vec![0x02, reg, other],
            Sub { reg, other } => vec![0x03, reg, other],
            Read { reg, addr } => vec![0x04, reg, addr],
            Write { reg, addr } => vec![0x05, reg, addr],
            Jump { addr } => vec![0x06, addr],
            JumpIfNotZero { reg, addr } => vec![0x07, reg, addr],
            Print { reg } => vec![0x08, reg],
        }
    }
}

/// Assembly language: the same instruction, written for people.
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Halt => write!(f, "HALT"),
            Load { reg, value } => write!(f, "LOAD  R{reg}, {value}"),
            Add { reg, other } => write!(f, "ADD   R{reg}, R{other}"),
            Sub { reg, other } => write!(f, "SUB   R{reg}, R{other}"),
            Read { reg, addr } => write!(f, "READ  R{reg}, [{addr}]"),
            Write { reg, addr } => write!(f, "WRITE R{reg}, [{addr}]"),
            Jump { addr } => write!(f, "JUMP  {addr}"),
            JumpIfNotZero { reg, addr } => write!(f, "JNZ   R{reg}, {addr}"),
            Print { reg } => write!(f, "PRINT R{reg}"),
        }
    }
}

#[derive(Debug, PartialEq)]
enum CpuError {
    UnknownOpcode { at: u8, opcode: u8 },
    BadRegister { at: u8, reg: u8 },
    TooManySteps,
}

struct Cpu {
    memory: [u8; 256],
    registers: [u8; 4],
    pc: u8,
    output: Vec<u8>,
}

impl Cpu {
    fn new(memory: [u8; 256]) -> Self {
        Cpu {
            memory,
            registers: [0; 4],
            pc: 0,
            output: Vec::new(),
        }
    }

    /// The byte `offset` places after PC. Addresses are one byte, so they wrap
    /// around from 255 to 0, and can never point outside the 256 bytes.
    fn byte_at(&self, offset: u8) -> u8 {
        self.memory[self.pc.wrapping_add(offset) as usize]
    }

    /// Steps 1 and 2: FETCH the bytes at PC, and DECODE them.
    fn fetch_and_decode(&self) -> Result<Instruction, CpuError> {
        let (opcode, a, b) = (self.byte_at(0), self.byte_at(1), self.byte_at(2));
        let reg = |r: u8| {
            if r < 4 {
                Ok(r)
            } else {
                Err(CpuError::BadRegister {
                    at: self.pc,
                    reg: r,
                })
            }
        };
        Ok(match opcode {
            0x00 => Halt,
            0x01 => Load {
                reg: reg(a)?,
                value: b,
            },
            0x02 => Add {
                reg: reg(a)?,
                other: reg(b)?,
            },
            0x03 => Sub {
                reg: reg(a)?,
                other: reg(b)?,
            },
            0x04 => Read {
                reg: reg(a)?,
                addr: b,
            },
            0x05 => Write {
                reg: reg(a)?,
                addr: b,
            },
            0x06 => Jump { addr: a },
            0x07 => JumpIfNotZero {
                reg: reg(a)?,
                addr: b,
            },
            0x08 => Print { reg: reg(a)? },
            opcode => {
                return Err(CpuError::UnknownOpcode {
                    at: self.pc,
                    opcode,
                });
            }
        })
    }

    /// Step 3: EXECUTE one instruction. Returns false after HALT.
    fn execute(&mut self, instruction: Instruction) -> bool {
        let next = self.pc.wrapping_add(instruction.encoded_len());
        let r = &mut self.registers;
        match instruction {
            Halt => return false,
            Load { reg, value } => r[reg as usize] = value,
            Add { reg, other } => r[reg as usize] = r[reg as usize].wrapping_add(r[other as usize]),
            Sub { reg, other } => r[reg as usize] = r[reg as usize].wrapping_sub(r[other as usize]),
            Read { reg, addr } => r[reg as usize] = self.memory[addr as usize],
            Write { reg, addr } => self.memory[addr as usize] = r[reg as usize],
            Print { reg } => self.output.push(r[reg as usize]),
            Jump { addr } => {
                self.pc = addr;
                return true;
            }
            JumpIfNotZero { reg, addr } => {
                self.pc = if r[reg as usize] != 0 { addr } else { next };
                return true;
            }
        }
        self.pc = next;
        true
    }

    /// Runs until HALT, at most `max_steps` instructions (a program might loop
    /// forever). `trace` is called before every instruction.
    fn run(
        &mut self,
        max_steps: usize,
        mut trace: impl FnMut(&Cpu, Instruction),
    ) -> Result<usize, CpuError> {
        for step in 1..=max_steps {
            let instruction = self.fetch_and_decode()?;
            trace(self, instruction);
            if !self.execute(instruction) {
                return Ok(step);
            }
        }
        Err(CpuError::TooManySteps)
    }
}

/// Puts a program at address 0 of a fresh memory. Returns the memory and the
/// address where each instruction starts.
fn assemble(program: &[Instruction]) -> ([u8; 256], Vec<u8>) {
    assert!(
        program
            .iter()
            .map(|i| usize::from(i.encoded_len()))
            .sum::<usize>()
            <= 256,
        "program must fit in 256-byte memory"
    );
    let mut memory = [0u8; 256];
    let mut starts = Vec::new();
    let mut address = 0usize;
    for instruction in program {
        starts.push(address as u8);
        for byte in instruction.encode() {
            memory[address] = byte;
            address += 1;
        }
    }
    (memory, starts)
}

/// 6 × 7, by adding 6 seven times: this CPU has no multiply instruction.
/// The numbers are in memory at addresses 100 and 101; the answer goes to 102.
fn multiply_program() -> Vec<Instruction> {
    vec![
        Read { reg: 0, addr: 100 },         // 0:  R0 = memory[100]   (6)
        Read { reg: 1, addr: 101 },         // 3:  R1 = memory[101]   (7, the counter)
        Load { reg: 2, value: 0 },          // 6:  R2 = 0            (the result)
        Load { reg: 3, value: 1 },          // 9:  R3 = 1
        Add { reg: 2, other: 0 },           // 12: R2 = R2 + R0      ← the loop starts here
        Sub { reg: 1, other: 3 },           // 15: R1 = R1 - 1
        JumpIfNotZero { reg: 1, addr: 12 }, // 18: if R1 != 0, go back to 12
        Write { reg: 2, addr: 102 },        // 21: memory[102] = R2
        Print { reg: 2 },                   // 24: show R2
        Halt,                               // 26: stop
    ]
}

fn main() {
    let (mut memory, starts) = assemble(&multiply_program());
    memory[100] = 6;
    memory[101] = 7;

    println!("1. The program, as people write it and as the CPU reads it");
    for (instruction, start) in multiply_program().iter().zip(&starts) {
        let bytes: Vec<String> = instruction
            .encode()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        println!("    {start:>3}: {:<9} {}", bytes.join(" "), instruction);
    }
    println!(
        "    the data at 100 and 101: {} and {}",
        memory[100], memory[101]
    );

    println!("\n2. Running it, one instruction at a time");
    println!("     PC  instruction       R0  R1  R2  R3");
    let mut cpu = Cpu::new(memory);
    let result = cpu.run(1_000, |cpu, instruction| {
        let [r0, r1, r2, r3] = cpu.registers;
        println!(
            "    {:>3}  {:<16} {r0:>3} {r1:>3} {r2:>3} {r3:>3}",
            cpu.pc,
            instruction.to_string()
        );
    });

    println!("\n3. The result");
    match result {
        Ok(steps) => println!(
            "    halted after {steps} instructions; printed {:?}; memory[102] = {}",
            cpu.output, cpu.memory[102]
        ),
        Err(error) => println!("    the CPU stopped with an error: {error:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(memory: [u8; 256]) -> (Cpu, Result<usize, CpuError>) {
        let mut cpu = Cpu::new(memory);
        let result = cpu.run(1_000, |_, _| {});
        (cpu, result)
    }

    #[test]
    fn multiplies_by_repeated_addition() {
        for (a, b) in [(6, 7), (1, 1), (12, 10), (3, 1)] {
            let (mut memory, _) = assemble(&multiply_program());
            memory[100] = a;
            memory[101] = b;
            let (cpu, result) = run(memory);
            assert!(result.is_ok());
            assert_eq!(cpu.memory[102], a * b);
            assert_eq!(cpu.output, [a * b]);
        }
    }

    #[test]
    fn encoding_then_decoding_gives_the_same_instruction() {
        for instruction in multiply_program() {
            let mut memory = [0u8; 256];
            memory[..instruction.encode().len()].copy_from_slice(&instruction.encode());
            assert_eq!(Cpu::new(memory).fetch_and_decode(), Ok(instruction));
        }
    }

    #[test]
    fn arithmetic_wraps_like_a_real_8_bit_register() {
        let (memory, _) = assemble(&[
            Load { reg: 0, value: 250 },
            Load { reg: 1, value: 10 },
            Add { reg: 0, other: 1 },
            Halt,
        ]);
        let (cpu, _) = run(memory);
        assert_eq!(cpu.registers[0], 4); // 260 doesn't fit in a byte: 260 - 256
    }

    #[test]
    fn bad_machine_code_is_an_error_not_a_crash() {
        let mut memory = [0u8; 256];
        memory[0] = 0xFF;
        assert_eq!(
            run(memory).1,
            Err(CpuError::UnknownOpcode {
                at: 0,
                opcode: 0xFF
            })
        );
        memory[0..3].copy_from_slice(&[0x01, 9, 1]); // LOAD R9: there is no R9
        assert_eq!(run(memory).1, Err(CpuError::BadRegister { at: 0, reg: 9 }));
    }

    #[test]
    fn an_endless_loop_is_stopped() {
        let (memory, _) = assemble(&[Jump { addr: 0 }]);
        assert_eq!(run(memory).1, Err(CpuError::TooManySteps));
    }

    #[test]
    fn a_program_can_overwrite_itself() {
        // Code and data share one memory: this program changes its own next
        // instruction from HALT (00) into PRINT R0 (08 00) before reaching it.
        let (mut memory, _) = assemble(&[
            Load { reg: 1, value: 8 },
            Write { reg: 1, addr: 9 },
            Load { reg: 0, value: 42 },
            Halt,
        ]);
        memory[10] = 0; // operand of the PRINT to be: R0
        memory[11] = 0x00; // and a HALT after it
        let (cpu, _) = run(memory);
        assert_eq!(cpu.output, [42]);
    }
}
