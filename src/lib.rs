#![feature(const_convert)]
#![feature(const_mut_refs)]
#![feature(const_option_ext)]
#![feature(const_trait_impl)]
#![feature(const_try)]

use core::ops;
use encoder::Encoder;
use pancake::Vec;

pub use reg::Reg;

mod encoder;
mod reg;

/// An instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Inst {
    Call(i32),
    Call2(i32),
    Lea(Reg, Arg),
    Mov(Reg, Arg),
    Pop(Reg),
    Push(Arg),
    Jmp(i32),
    Ret,
    Syscall,
    Xor(Reg, Reg),
}

const REX_W: u8 = 0x48;

impl Inst {
    #[inline]
    pub const fn from_bytes(bytes: &[u8]) -> Option<Inst> {
        let inst = match bytes {
            /*// mov rax rdi
            [REX_W, 0x89, 0xC7, ..] => {}
            // xor rdi rdi
            [REX_W, 0x31, 0xFF, ..] => {}
            // mov rdi, i32
            [REX_W, 0xC7, 0xC7, a, b, c, d, ..] => {}
            // mov rax, i32
            [REX_W, 0xC7, 0xC0, a, b, c, d, ..] => {}
            // lea rip rsi
            [REX_W, 0x8D, 0x35, a, b, c, d, ..] => {}*/

            // mov rax, qword ptr [rip+rel]
            [REX_W, 0x8B, 0x05, a, b, c, d, ..] => {
                Inst::Mov(Reg::Rax, Arg::Int(i32::from_le_bytes([*a, *b, *c, *d])))
            }

            // lea
            [REX_W, 0x8D, 0x0D, a, b, c, d, ..] => {
                Inst::Lea(Reg::Rcx, Arg::Int(i32::from_le_bytes([*a, *b, *c, *d])))
            }

            // jmp
            [0xFF, 0x25, a, b, c, d, ..] => Inst::Jmp(i32::from_le_bytes([*a, *b, *c, *d])),

            // call
            [0xFF, 0x15, a, b, c, d, ..] => Inst::Call2(i32::from_le_bytes([*a, *b, *c, *d])),

            // call
            [0xE8, a, b, c, d, ..] => Inst::Call(i32::from_le_bytes([*a, *b, *c, *d])),

            // push
            [0x6A, byte] => Inst::Push(Arg::Int(*byte as i32)),

            // syscall
            [0x0F, 0x05, ..] => Inst::Syscall,

            // push reg <= r7
            [0x41, reg @ 0x50..=0x57, ..] => {
                Inst::Push(Arg::Reg(unsafe { Reg::from_hi_unchecked(*reg) }))
            }

            // pop reg <= r7
            [0x41, reg @ 0x58..=0x5F, ..] => Inst::Pop(unsafe { Reg::from_hi_unchecked(*reg) }),

            // ret
            [0xC3, ..] => Inst::Ret,

            // push reg <= r7
            [reg @ 0x50..=0x57, ..] => {
                Inst::Push(Arg::Reg(unsafe { Reg::from_lo_unchecked(*reg) }))
            }

            // pop reg <= r7
            [reg @ 0x58..=0x5F, ..] => Inst::Pop(unsafe { Reg::from_lo_unchecked(*reg) }),

            _ => return None,
        };

        Some(inst)
    }

    #[inline]
    pub const fn to_bytes(&self) -> Vec<u8, 15> {
        let mut encoder = Encoder::new();

        unsafe {
            match *self {
                Inst::Call(rel) => {
                    encoder.write_u8(0xE8);
                    encoder.write_i32(rel);
                }
                Inst::Call2(rel) => {
                    encoder.write_bytes(&[0xFF, 0x15]);
                    encoder.write_i32(rel);
                }
                Inst::Lea(Reg::Rcx, Arg::Int(rel)) => {
                    encoder.write_bytes(&[REX_W, 0x8D, 0x0D]);
                    encoder.write_i32(rel);
                }
                Inst::Jmp(rel) => {
                    encoder.write_bytes(&[0xFF, 0x25]);
                    encoder.write_i32(rel);
                }
                Inst::Mov(Reg::Rax, Arg::Int(rel)) => {
                    encoder.write_bytes(&[REX_W, 0x8B, 0x05]);
                    encoder.write_i32(rel);
                }
                Inst::Pop(reg) => {
                    if reg.is_hi() {
                        encoder.write_bytes(&[0x41, 0x58 | reg.base_bits()]);
                    } else {
                        encoder.write_u8(0x58 | reg.bits());
                    }
                }
                Inst::Push(Arg::Reg(reg)) => {
                    if reg.is_hi() {
                        encoder.write_bytes(&[0x41, 0x50 | reg.base_bits()]);
                    } else {
                        encoder.write_u8(0x50 | reg.bits());
                    }
                }
                Inst::Ret => {
                    encoder.write_u8(0xC3);
                }
                Inst::Syscall => {
                    encoder.write_bytes(&[0x0F, 0x05]);
                }
                _ => unreachable!(),
            }
        }

        encoder.into_vec()
    }

    /// Returns the relative address if present in this instruction.
    #[inline]
    pub const fn rel_addr(&self) -> Option<isize> {
        let rel = match self {
            Inst::Call(rel)
            | Inst::Call2(rel)
            | Inst::Jmp(rel)
            | Inst::Mov(Reg::Rax, Arg::Int(rel)) => rel,
            _ => return None,
        };

        Some(*rel as isize)
    }
    /// obtains the length of the instruction (max 15)
    #[inline]
    pub const fn len(&self) -> usize {
        match self {
            Inst::Call(_) => 5,
            Inst::Call2(_) => 6,
            Inst::Lea(_, _) => 7,
            Inst::Jmp(_) => 6,
            Inst::Mov(_, _) => 7,
            Inst::Pop(reg) => {
                if reg.is_hi() {
                    2
                } else {
                    1
                }
            }
            Inst::Push(_) => 1,
            Inst::Ret => 1,
            Inst::Syscall => 2,
            Inst::Xor(_, _) => 3,
        }
    }
}

/// A register or i32,
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arg {
    Reg(Reg),
    Int(i32),
}

/// Instruction pointer alongside an instruction.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WithIp {
    ip: usize,
    inst: Inst,
}

impl WithIp {
    /// Construct a new instruction with an associated instruction pointer.
    #[inline]
    pub const fn new(ip: usize, inst: Inst) -> Self {
        Self { ip, inst }
    }

    /// Resolves the relative address (if present)
    #[inline]
    pub const fn abs_addr(self) -> Option<usize> {
        // relative addresses are calculated from the ip after the current instruction.
        let ip = self.next_ip();
        let addr = ip as isize + self.inst.rel_addr()?;

        Some(addr as usize)
    }

    /// Returns the current instruction pointer.
    #[inline]
    pub const fn ip(self) -> usize {
        self.ip
    }

    /// Returns the next instruction pointer.
    ///
    /// Equivalent to `withip.ip() + withip.len()`.
    #[inline]
    pub const fn next_ip(self) -> usize {
        self.ip + self.inst.len()
    }

    /// Fancy formatter, not implemented yet.
    #[inline]
    pub const fn display(self) -> Inst {
        self.inst
    }
}

impl ops::Deref for WithIp {
    type Target = Inst;

    #[inline]
    fn deref(&self) -> &Inst {
        &self.inst
    }
}

/// Instruction iterator (decoder).
pub struct InstIter<'a> {
    bytes: &'a [u8],
    ip: usize,
    offset: usize,
}

impl<'a> InstIter<'a> {
    #[inline]
    pub fn from_bytes(ip: usize, bytes: &'a [u8]) -> Self {
        let offset = 0;

        Self { bytes, ip, offset }
    }
}

impl<'a> Iterator for InstIter<'a> {
    type Item = WithIp;

    #[inline]
    fn next(&mut self) -> Option<WithIp> {
        let rest = &self.bytes[self.offset..];

        match Inst::from_bytes(rest) {
            Some(inst) => {
                let ip = self.ip + self.offset;

                self.offset += inst.len();

                Some(WithIp::new(ip, inst))
            }
            None => None,
        }
    }
}
