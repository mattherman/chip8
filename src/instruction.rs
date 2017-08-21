pub type Register = u8;
pub type Address = u16;
pub type Value = u8;

use std::{ fmt };

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Instruction {
    SetIndexRegister(Address),
    Random(Register, Value),
    Draw(Register, Register, Value),
    Jump(Address),
    SkipIfEqual(Register, Value),
    Load(Register, Value),
    Add(Register, Value),
    Assign(Register, Register),
    InvalidOperation,
}

impl Instruction {
    pub fn parse(val: u16) -> Instruction {
        match val & 0xF000 {
            0xA000 => Instruction::SetIndexRegister(get_address(val)),
            0xC000 => Instruction::Random(get_first_register(val), get_value(val)),
            0xD000 => Instruction::Draw(get_first_register(val), get_second_register(val), get_value(val)),
            0x1000 => Instruction::Jump(get_address(val)),
            0x3000 => Instruction::SkipIfEqual(get_first_register(val), get_value(val)),
            0x6000 => Instruction::Load(get_first_register(val), get_value(val)),
            0x7000 => Instruction::Add(get_first_register(val), get_value(val)),
            0x8000 => match val & 0x000F {
                0x0000 => Instruction::Assign(get_first_register(val), get_second_register(val)),
                _ => Instruction::InvalidOperation,
            },
            _ => Instruction::InvalidOperation,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pretty_instruction = match *self {
            Instruction::Load(r, v) => format!("LD V{}, {}", r, v),
            Instruction::SetIndexRegister(a) => format!("LD I, 0x{:X}", a),
            Instruction::Random(r, v) => format!("RND V{}, {}", r, v),
            Instruction::Jump(a) => format!("JP 0x{:X}", a),
            Instruction::SkipIfEqual(r, v) => format!("SE V{}, {}", r, v),
            Instruction::Draw(r1, r2, v) => format!("DRW V{}, V{}, {}", r1, r2, v),
            Instruction::Add(r, v) => format!("ADD V{}, {}", r, v),
            Instruction::Assign(r1, r2) => format!("LD V{}, V{}", r1, r2),
            Instruction::InvalidOperation => format!("INVALID OPERATION"),
        };
        write!(f, "{}", pretty_instruction)
    }
}

fn get_first_register(val: u16) -> u8 {
    ((val & 0x0F00) >> 8) as u8
}

fn get_second_register(val: u16) -> u8 {
    ((val & 0x00F0) >> 4) as u8
}

fn get_value(val: u16) -> u8 {
    (val & 0x00FF) as u8
}

fn get_address(val: u16) -> u16 {
    (val & 0x0FFF)
}

// Executing: 0x6000
// Executing: 0x6100
// Executing: 0xA222
// Executing: 0xC201
// Executing: 0x3201
// Executing: 0xA21E
// Executing: 0xD014
// Executing: 0x7004
// Executing: 0x3040
// Executing: 0x1204
// Executing: 0x6000
// Executing: 0x7104
// Executing: 0x3120
// Executing: 0x1204
// Executing: 0x121C
// Executing: 0x8040
// Executing: 0x2010
// Executing: 0x2040
// Executing: 0x8010
