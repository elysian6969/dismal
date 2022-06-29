#![feature(const_convert)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(const_try)]

use core::ops;
use pancake::Vec;

pub use reg::Reg;

mod reg;

/// An instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Inst {
    Call(i32),
    Lea(Reg, Arg),
    Pop(Reg),
    Push(Arg),
    Ret,
    Syscall,
    Xor(Reg, Reg),
}

const REX_W: u8 = 0x48;

#[inline]
const fn try_parse_pop_one(byte: u8) -> Option<Reg> {
    Reg::try_parse(byte)
}

#[inline]
const fn try_parse_pop_two(byte: u8) -> Option<Reg> {
    Reg::try_parse(byte & !0b1000)
}

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
            [REX_W, 0x8D, 0x0D, a, b, c, d, ..] => {
                Inst::Lea(Reg::Rcx, Arg::Int(i32::from_le_bytes([*a, *b, *c, *d])))
            }
            [0xFF, 0x15, a, b, c, d, ..] => Inst::Call(i32::from_le_bytes([*a, *b, *c, *d])),
            // push
            [0x50, ..] => Inst::Push(Arg::Reg(Reg::Rax)),
            [0x6A, byte] => Inst::Push(Arg::Int(*byte as i32)),

            // syscall
            [0x0F, 0x05, ..] => Inst::Syscall,

            // pop two
            [0x41, reg, ..] => Inst::Pop(try_parse_pop_two(*reg)?),

            // ret
            [0xC3, ..] => Inst::Ret,

            // maybe pop one
            [unk, ..] => Inst::Pop(try_parse_pop_one(*unk)?),

            _ => return None,
        };

        Some(inst)
    }

    #[inline]
    pub const fn to_bytes(&self) -> Vec<u8, 15> {
        let mut bytes = Vec::new();

        unsafe {
            match self {
                Inst::Call(rel) => {
                    bytes.extend_from_slice_unchecked(&[0xFF, 0x15]);
                    bytes.extend_from_slice_unchecked(&rel.to_le_bytes());
                }
                Inst::Lea(Reg::Rcx, Arg::Int(rel)) => {
                    bytes.extend_from_slice_unchecked(&[REX_W, 0x8D, 0x0D]);
                    bytes.extend_from_slice_unchecked(&rel.to_le_bytes());
                }
                Inst::Pop(reg) => {
                    if reg.is_hi() {
                        bytes.extend_from_slice_unchecked(&[0x41, 0x58 | reg.base_bits()]);
                    } else {
                        bytes.push_unchecked(0x58 | reg.bits());
                    }
                }
                Inst::Push(Arg::Reg(Reg::Rax)) => {
                    bytes.push_unchecked(0x50);
                }
                Inst::Ret => {
                    bytes.push_unchecked(0xC3);
                }
                Inst::Syscall => {
                    bytes.extend_from_slice_unchecked(&[0x0F, 0x05]);
                }
                _ => unreachable!(),
            }
        }

        bytes
    }

    /// obtains the length of the instruction (max 15)
    #[inline]
    pub const fn len(&self) -> usize {
        match self {
            Inst::Call(_) => 6,
            Inst::Lea(_, _) => 7,
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WithIp {
    ip: usize,
    inst: Inst,
}

impl WithIp {
    #[inline]
    pub const fn ip(&self) -> usize {
        self.ip
    }

    #[inline]
    pub const fn display(&self) -> Inst {
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

                Some(WithIp { ip, inst })
            }
            None => None,
        }
    }
}

fn test(bytes: &[u8]) {
    println!("---");
    println!("bytes = {bytes:02X?}");

    if let Some(inst) = Inst::from_bytes(bytes) {
        println!("inst = {inst:0X?}");
        println!("reenc = {:02X?}", inst.to_bytes());
    } else {
        println!("failed to decode");
    }

    println!("---");
}

fn main() {
    test(&[0x50]);

    test(&[0xFF, 0x15, 0x69, 0x69, 0x69, 0x69]);

    test(&[0x58]);

    test(&[0xC3]);

    test(&[0x41, 0x5F]);
}
