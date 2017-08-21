use instruction::{ Instruction, Register, Address, Value };
use rand;
use rand::Rng;

const INSTRUCTION_SIZE: u16 = 2;

pub struct Cpu {
    memory: [u8; 4096],
    registers: [u8; 16],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u16,
    pub draw_flag: bool,
    pub faulted: bool,
}

impl Cpu {
    pub fn new(game_data: Vec<u8>) -> Cpu {
        let mut memory = [0; 4096];
        for (i, byte) in game_data.iter().enumerate() {
            memory[0x200 + i] = byte.clone();
        }

        Cpu {
            memory: memory,
            registers: [0; 16],
            index: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            draw_flag: false,
            faulted: false,
        }
    }

    pub fn cycle(&mut self) {
        if self.faulted {
            return
        }

        let raw_instruction = self.read_next_instruction();
        let instruction = Instruction::parse(raw_instruction);

        println!("[PC:0x{:X}] [RAW:0x{:X}] {}", self.pc, raw_instruction, instruction);

        if instruction == Instruction::InvalidOperation {
            self.faulted = true;
        } else {
            self.execute_instruction(instruction);
            self.print_registers();
        }
    }

    fn read_next_instruction(&self) -> (u16) {
        let upper = self.memory[self.pc as usize] as u16;
        let lower = self.memory[(self.pc + 1) as usize] as u16;
        upper << 8 | lower
    }

    fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Load(r, v) => self.load_value(r, v),
            Instruction::SetIndexRegister(a) => self.set_index(a),
            Instruction::Random(r, v) => self.rand(r, v),
            Instruction::Jump(a) => self.jump(a),
            Instruction::SkipIfEqual(r, v) => self.skip_equal(r, v),
            Instruction::Draw(r1, r2, v) => self.skip(),
            Instruction::Add(r, v) => self.add(r, v),
            Instruction::Assign(r1, r2) => self.assign(r1, r2),
            Instruction::InvalidOperation => {},
        };
    }

    fn read_register(&self, register: Register) -> Value {
        self.registers[register as usize]
    }

    fn set_register(&mut self, register: Register, value: Value) {
        self.registers[register as usize] = value;
    }

    fn skip(&mut self) {
        self.pc += 2;
    }

    fn load_value(&mut self, register: Register, value: Value) {
        self.set_register(register, value);
        self.pc += INSTRUCTION_SIZE;
    }

    fn add(&mut self, register: Register, value: Value) {
        let current_value = self.read_register(register);
        self.set_register(register, current_value + value);
        self.pc += INSTRUCTION_SIZE;
    }

    fn set_index(&mut self, addr: Address) {
        self.index = addr;
        self.pc += INSTRUCTION_SIZE;
    }

    fn jump(&mut self, addr: Address) {
        self.pc = addr;
    }

    fn skip_equal(&mut self, register: Register, value: Value) {
        let reg_val = self.read_register(register);
        let pc_skip = if reg_val == value {
            INSTRUCTION_SIZE * 2
        } else {
            INSTRUCTION_SIZE
        };
        self.pc += pc_skip;
    }

    fn assign(&mut self, register1: Register, register2: Register) {
        let reg2_val = self.read_register(register2);
        self.set_register(register1, reg2_val);
        self.pc += 2;
    }

    fn rand(&mut self, register: Register, value: Value) {
        let rand_val = rand::thread_rng().gen::<u8>();
        self.set_register(register, value & rand_val);
        self.pc += 2;
    }

    fn print_registers(&self) {
        let reg = self.registers;
        println!("V0:{} V1:{} V2:{} V3:{} V4:{} V5:{} V6:{} V7:{}", 
            reg[0], reg[1], reg[2], reg[3], reg[4], reg[5], reg[6], reg[7]);
        println!("V8:{} V9:{} VA:{} VB:{} VC:{} VD:{} VE:{} VF:{}", 
            reg[8], reg[9], reg[10], reg[11], reg[12], reg[13], reg[14], reg[15]);
    }
}
