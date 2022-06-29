// ty https://wiki.osdev.org/X86-64_Instruction_Encoding

const REG_MASK: u8 = 0b0000_0111;

const HI_BIT: u8 = 0b1000;

const R0: u8 = 0b000;
const R1: u8 = 0b001;
const R2: u8 = 0b010;
const R3: u8 = 0b011;
const R4: u8 = 0b100;
const R5: u8 = 0b101;
const R6: u8 = 0b110;
const R7: u8 = 0b111;

/// A register.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reg {
    Rax,
    Rcx,
    Rdx,
    Rbx,
    Rsp,
    Rbp,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Reg {
    /// Attempt to parse a register.
    #[inline]
    pub const fn try_parse(reg: u8) -> Option<Self> {
        let is_lo = reg & HI_BIT != 0;
        let reg = reg & REG_MASK;

        if is_lo {
            from_lo(reg)
        } else {
            from_hi(reg)
        }
    }

    #[inline]
    pub const fn is_lo(self) -> bool {
        !self.is_hi()
    }

    #[inline]
    pub const fn is_hi(self) -> bool {
        matches!(
            self,
            Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11 | Reg::R12 | Reg::R13 | Reg::R14 | Reg::R15
        )
    }

    #[inline]
    pub const fn base_bits(self) -> u8 {
        if self.is_hi() {
            to_hi(self)
        } else {
            to_lo(self)
        }
    }

    #[inline]
    pub const fn bits(self) -> u8 {
        if self.is_hi() {
            to_hi(self) | HI_BIT
        } else {
            to_lo(self)
        }
    }
}

#[inline]
const fn from_lo(bits: u8) -> Option<Reg> {
    let reg = match bits {
        R0 => Reg::Rax,
        R1 => Reg::Rcx,
        R2 => Reg::Rdx,
        R3 => Reg::Rbx,
        R4 => Reg::Rsp,
        R5 => Reg::Rbp,
        R6 => Reg::Rsi,
        R7 => Reg::Rdi,
        _ => return None,
    };

    Some(reg)
}

#[inline]
const fn from_hi(byte: u8) -> Option<Reg> {
    let reg = match byte {
        R0 => Reg::R8,
        R1 => Reg::R9,
        R2 => Reg::R10,
        R3 => Reg::R11,
        R4 => Reg::R12,
        R5 => Reg::R13,
        R6 => Reg::R14,
        R7 => Reg::R15,
        _ => return None,
    };

    Some(reg)
}

#[inline]
const fn to_lo(reg: Reg) -> u8 {
    let bits = match reg {
        Reg::Rax => R0,
        Reg::Rcx => R1,
        Reg::Rdx => R2,
        Reg::Rbx => R3,
        Reg::Rsp => R4,
        Reg::Rbp => R5,
        Reg::Rsi => R6,
        Reg::Rdi => R7,
        _ => unreachable!(),
    };

    bits
}

#[inline]
const fn to_hi(reg: Reg) -> u8 {
    let bits = match reg {
        Reg::R8 => R0,
        Reg::R9 => R1,
        Reg::R10 => R2,
        Reg::R11 => R3,
        Reg::R12 => R4,
        Reg::R13 => R5,
        Reg::R14 => R6,
        Reg::R15 => R7,
        _ => unreachable!(),
    };

    bits
}
