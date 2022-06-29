use core::ops;

/// An instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Inst {
    Call(i32),
    Lea(Reg, i32),
    Pop(Reg),
    Push(Reg),
    Ret,
}

impl Inst {
    /// obtains the length of the instruction (max 15)
    pub fn len(&self) -> usize {
        match self {
            Inst::Call(_) => 6,
            Inst::Lea(_, _) => 7,
            Inst::Pop(_) => 1,
            Inst::Push(_) => 1,
            Inst::Ret => 1,
        }
    }
}

/// A register,
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reg {
    Rax,
    Rcx,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WithIp {
    ip: usize,
    inst: Inst,
}

impl WithIp {
    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn display(&self) -> Inst {
        self.inst
    }
}

impl ops::Deref for WithIp {
    type Target = Inst;

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

        match try_parse(rest) {
            Some(inst) => {
                let ip = self.ip + self.offset;

                self.offset += inst.len();

                Some(WithIp { ip, inst })
            }
            None => None,
        }
    }
}

#[inline]
fn try_parse(bytes: &[u8]) -> Option<Inst> {
    let inst = match bytes {
        [0x48, 0x8D, 0x0D, a, b, c, d, ..] => {
            Inst::Lea(Reg::Rcx, i32::from_le_bytes([*a, *b, *c, *d]))
        }
        [0xFF, 0x15, a, b, c, d, ..] => Inst::Call(i32::from_le_bytes([*a, *b, *c, *d])),
        [0x50, ..] => Inst::Push(Reg::Rax),
        [0x58, ..] => Inst::Pop(Reg::Rax),
        [0xC3, ..] => Inst::Ret,
        _ => return None,
    };

    Some(inst)
}

fn main() {
    let insts = InstIter::from_bytes(
        0x1000,
        &[0x50, 0xFF, 0x15, 0x69, 0x69, 0x69, 0x69, 0x58, 0xC3],
    );

    for inst in insts {
        println!("{:0X?} {:?}", inst.ip(), inst.display());
    }
}
