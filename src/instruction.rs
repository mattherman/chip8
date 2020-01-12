pub type Register = u8;
pub type Address = u16;
pub type Value = u8;

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Clear,
    Return,
    ExRoutine(Address),
    Jump(Address),
    Call(Address),
    SkipIfEqual(Register, Value),
    SkipIfNotEqual(Register, Value),
    SkipIfRegEqual(Register, Register),
    LoadVal(Register, Value),
    AddVal(Register, Value),
    LoadReg(Register, Register),
    Or(Register, Register),
    And(Register, Register),
    Xor(Register, Register),
    AddReg(Register, Register),
    SubReg(Register, Register),
    ShiftRight(Register),
    ShiftLeft(Register),
    SetIndexRegister(Address),
    Random(Register, Value),
    Draw(Register, Register, Value),
    SkipIfKey(Register),
    SkipIfNotKey(Register),
    AddIndex(Register),
    LoadDigit(Register),
    LoadBCD(Register),
    StoreIndex(Register),
    ReadIndex(Register),
    InvalidOperation,
}

impl Instruction {
    pub fn parse(val: u16) -> Instruction {
        match val & 0xF000 {
            0x0000 => match val & 0x0FFF {
                0x00E0 => Instruction::Clear,
                0x00EE => Instruction::Return,
                _ => Instruction::ExRoutine(addr(val)),
            },
            0x1000 => Instruction::Jump(addr(val)),
            0x2000 => Instruction::Call(addr(val)),
            0x3000 => Instruction::SkipIfEqual(reg1(val), byte(val)),
            0x4000 => Instruction::SkipIfNotEqual(reg1(val), byte(val)),
            0x5000 => Instruction::SkipIfRegEqual(reg1(val), reg2(val)),
            0x6000 => Instruction::LoadVal(reg1(val), byte(val)),
            0x7000 => Instruction::AddVal(reg1(val), byte(val)),
            0x8000 => match val & 0x000F {
                0x0000 => Instruction::LoadReg(reg1(val), reg2(val)),
                0x0001 => Instruction::Or(reg1(val), reg2(val)),
                0x0002 => Instruction::And(reg1(val), reg2(val)),
                0x0003 => Instruction::Xor(reg1(val), reg2(val)),
                0x0004 => Instruction::AddReg(reg1(val), reg2(val)),
                0x0005 => Instruction::SubReg(reg1(val), reg2(val)),
                0x0006 => Instruction::ShiftRight(reg1(val)),
                0x000E => Instruction::ShiftLeft(reg1(val)),
                _ => Instruction::InvalidOperation,
            },
            0xA000 => Instruction::SetIndexRegister(addr(val)),
            0xC000 => Instruction::Random(reg1(val), byte(val)),
            0xD000 => Instruction::Draw(reg1(val), reg2(val), nibble(val)),
            0xE000 => match val & 0x00FF {
                0x009E => Instruction::SkipIfKey(reg1(val)),
                0x00A1 => Instruction::SkipIfNotKey(reg1(val)),
                _ => Instruction::InvalidOperation,
            },
            0xF000 => match val & 0x00FF {
                0x001E => Instruction::AddIndex(reg1(val)),
                0x0029 => Instruction::LoadDigit(reg1(val)),
                0x0033 => Instruction::LoadBCD(reg1(val)),
                0x0055 => Instruction::StoreIndex(reg1(val)),
                0x0065 => Instruction::ReadIndex(reg1(val)),
                _ => Instruction::InvalidOperation,
            },
            _ => Instruction::InvalidOperation,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pretty_instruction = match *self {
            Instruction::Clear => format!("CLS"),
            Instruction::Return => format!("RET"),
            Instruction::ExRoutine(a) => format!("SYS 0x{:X}", a),
            Instruction::Jump(a) => format!("JP 0x{:X}", a),
            Instruction::Call(a) => format!("CALL 0x{:X}", a),
            Instruction::SkipIfEqual(r, v) => format!("SE V{:X}, {}", r, v),
            Instruction::SkipIfNotEqual(r, v) => format!("SNE V{:X}, {}", r, v),
            Instruction::SkipIfRegEqual(r1, r2) => format!("SE V{:X}, V{:X}", r1, r2),
            Instruction::LoadVal(r, v) => format!("LD V{:X}, {}", r, v),
            Instruction::AddVal(r, v) => format!("ADD V{:X}, {}", r, v),
            Instruction::LoadReg(r1, r2) => format!("LD V{:X}, V{:X}", r1, r2),
            Instruction::Or(r1, r2) => format!("OR V{:X}, V{:X}", r1, r2),
            Instruction::And(r1, r2) => format!("AND V{:X}, V{:X}", r1, r2),
            Instruction::Xor(r1, r2) => format!("XOR V{:X}, V{:X}", r1, r2),
            Instruction::AddReg(r1, r2) => format!("ADD V{:X}, V{:X}", r1, r2),
            Instruction::SubReg(r1, r2) => format!("SUB V{:X}, V{:X}", r1, r2),
            Instruction::ShiftRight(r) => format!("SHR V{:X}", r),
            Instruction::ShiftLeft(r) => format!("SHL V{:X}", r),
            Instruction::SetIndexRegister(a) => format!("LD I, 0x{:X}", a),
            Instruction::Random(r, v) => format!("RND V{:X}, {}", r, v),
            Instruction::Draw(r1, r2, v) => format!("DRW V{:X}, V{:X}, {}", r1, r2, v),
            Instruction::SkipIfKey(r) => format!("SKP V{:X}", r),
            Instruction::SkipIfNotKey(r) => format!("SKNP V{:X}", r),
            Instruction::AddIndex(r) => format!("ADD I, V{:X}", r),
            Instruction::LoadDigit(r) => format!("LD F, V{:X}", r),
            Instruction::LoadBCD(r) => format!("LD B, V{:X}", r),
            Instruction::StoreIndex(r) => format!("LD [I], V{:X}", r),
            Instruction::ReadIndex(r) => format!("LD V{:X} [I]", r),
            Instruction::InvalidOperation => format!("INVALID OPERATION"),
        };
        write!(f, "{}", pretty_instruction)
    }
}

fn reg1(val: u16) -> u8 {
    ((val & 0x0F00) >> 8) as u8
}

fn reg2(val: u16) -> u8 {
    ((val & 0x00F0) >> 4) as u8
}

fn byte(val: u16) -> u8 {
    (val & 0x00FF) as u8
}

fn nibble(val: u16) -> u8 {
    (val & 0x000F) as u8
}

fn addr(val: u16) -> u16 {
    (val & 0x0FFF)
}
