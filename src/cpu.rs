use instruction::{ Instruction, Register, Address, Value };
use display::{ Display, SPRITES, Screen };
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
    del_timer: u8,
    sound_timer: u8,
    pub display: Display,
    pub draw_flag: bool,
    pub faulted: bool,
}

impl Cpu {
    pub fn new(game_data: Vec<u8>) -> Cpu {
        let mut memory = [0; 4096];
        for (i, byte) in game_data.iter().enumerate() {
            memory[0x200 + i] = byte.clone();
        }
        for(i, byte) in SPRITES.iter().enumerate() {
            memory[i] = byte.clone();
        }

        let display = Display::new();

        Cpu {
            memory: memory,
            registers: [0; 16],
            index: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            del_timer: 0,
            sound_timer: 0,
            display: display,
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

        println!("[PC:0x{:X}] [RAW:0x{:04X}] {}", self.pc, raw_instruction, instruction);

        if instruction == Instruction::InvalidOperation {
            self.faulted = true;
        } else {
            self.execute_instruction(instruction);
            //self.print_registers();
            self.handle_timers();
        }
    }

    pub fn get_screen(&mut self) -> &Screen {
        self.display.get_screen()
    }

    fn read_next_instruction(&self) -> (u16) {
        let upper = self.memory[self.pc as usize] as u16;
        let lower = self.memory[(self.pc + 1) as usize] as u16;
        upper << 8 | lower
    }

    fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Clear => self.clear(),
            Instruction::Return => self.noop(),
            Instruction::Jump(a) => self.jump(a),
            Instruction::Call(a) => self.call(a),
            Instruction::Load(r, v) => self.load_value(r, v),            
            Instruction::SkipIfEqual(r, v) => self.skip_equal(r, v),
            Instruction::SkipIfNotEqual(r, v) => self.skip_not_equal(r, v),
            Instruction::Add(r, v) => self.add(r, v),
            Instruction::Assign(r1, r2) => self.assign(r1, r2),
            Instruction::SetIndexRegister(a) => self.set_index(a),
            Instruction::Random(r, v) => self.rand(r, v),
            Instruction::Draw(r1, r2, v) => self.draw(r1, r2, v),
            Instruction::AddStore(r) => self.add_store(r),
            Instruction::LoadDigit(r) => self.load_digit(r),
            Instruction::StoreRegisters(r) => self.store_reg(r),
            Instruction::ReadRegisters(r) => self.read_reg(r),
            Instruction::InvalidOperation => {},
        };
    }

    fn handle_timers(&mut self) {
        if self.del_timer > 0 {
            self.del_timer -= 1;
        }

        if self.sound_timer > 0 {
            // BEEP!
            self.sound_timer -= 1;
        }
    }

    fn read_register(&self, register: Register) -> Value {
        self.registers[register as usize]
    }

    fn set_register(&mut self, register: Register, value: Value) {
        self.registers[register as usize] = value;
    }

    fn set_program_counter(&mut self, new_addr: Address) {
        self.pc = new_addr;
    }

    fn noop(&mut self) {
        self.pc += INSTRUCTION_SIZE;
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
        self.set_program_counter(addr);
    }

    fn call(&mut self, addr: Address) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.set_program_counter(addr);
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

    fn skip_not_equal(&mut self, register: Register, value: Value) {
        let reg_val = self.read_register(register);
        let pc_skip = if reg_val != value {
            INSTRUCTION_SIZE * 2
        } else {
            INSTRUCTION_SIZE
        };
        self.pc += pc_skip;
    }

    fn assign(&mut self, register1: Register, register2: Register) {
        let reg2_val = self.read_register(register2);
        self.set_register(register1, reg2_val);
        self.pc += INSTRUCTION_SIZE;
    }

    fn rand(&mut self, register: Register, value: Value) {
        let rand_val = rand::thread_rng().gen::<u8>();
        self.set_register(register, value & rand_val);
        self.pc += INSTRUCTION_SIZE;
    }

    fn draw(&mut self, register1: Register, register2: Register, value: Value) {
        let x = self.read_register(register1);
        let y = self.read_register(register2);

        let mut sprite = Vec::new();
        for i in 0..(value-1) {
            let sprite_index = (self.index + i as u16) as usize;
            sprite.push(self.memory[sprite_index]);
        }

        self.display.draw_sprite(&sprite, x as usize, y as usize);

        self.pc += INSTRUCTION_SIZE;
    }

    fn clear(&mut self) {
        self.display.clear();

        self.pc += INSTRUCTION_SIZE;
    }

    fn add_store(&mut self, register: Register) {
        let current = self.memory[self.index as usize];
        self.memory[self.index as usize] = current + self.read_register(register);

        self.pc += INSTRUCTION_SIZE;
    }

    fn load_digit(&mut self, register: Register) {
        // Each digit sprite occupies five bytes of space starting at 0x00
        let sprite_location = 0x00 + (self.read_register(register) * 5);
        self.index = sprite_location as u16;

        self.pc += INSTRUCTION_SIZE;
    }

    fn store_reg(&mut self, register: Register) {
        for i in 0..(register+1) {
            let index = (self.index + i as u16) as usize;
            self.memory[index] = self.read_register(i);
        }

        self.pc += INSTRUCTION_SIZE;
    }

    fn read_reg(&mut self, register: Register) {
        for i in 0..(register+1) {
            let index = (self.index + i as u16) as usize;
            let new_val = self.memory[index];
            self.set_register(i, new_val);
        }

        self.pc += INSTRUCTION_SIZE;
    }

    fn print_registers(&self) {
        let reg = self.registers;
        println!("V0:{} V1:{} V2:{} V3:{} V4:{} V5:{} V6:{} V7:{}", 
            reg[0], reg[1], reg[2], reg[3], reg[4], reg[5], reg[6], reg[7]);
        println!("V8:{} V9:{} VA:{} VB:{} VC:{} VD:{} VE:{} VF:{}", 
            reg[8], reg[9], reg[10], reg[11], reg[12], reg[13], reg[14], reg[15]);
    }
}
