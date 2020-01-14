use display::{Display, Screen, SPRITES};
use instruction::{Address, Instruction, Register, Value};
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
    tick: u32,
    timer_tick: u32,
    keys: [bool; 16],
    debug_mode: bool,
    pub display: Display,
    pub draw_flag: bool,
    pub faulted: bool,
}

impl Cpu {
    pub fn new(game_data: Vec<u8>, clock_speed: u32, debug_mode: bool) -> Cpu {
        let mut memory = [0; 4096];
        for (i, byte) in game_data.iter().enumerate() {
            memory[0x200 + i] = byte.clone();
        }
        for (i, byte) in SPRITES.iter().enumerate() {
            memory[i] = byte.clone();
        }

        let display = Display::new(debug_mode);

        Cpu {
            memory: memory,
            registers: [0; 16],
            index: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            del_timer: 0,
            sound_timer: 0,
            tick: 0,
            timer_tick: clock_speed / 60,
            display: display,
            keys: [false; 16],
            draw_flag: false,
            faulted: false,
            debug_mode: debug_mode,
        }
    }

    pub fn cycle(&mut self) {
        if self.faulted {
            return;
        }

        let raw_instruction = self.read_next_instruction();
        let instruction = Instruction::parse(raw_instruction);

        println!(
            "[PC:0x{:X}] [RAW:0x{:04X}] {}",
            self.pc, raw_instruction, instruction
        );

        if instruction == Instruction::InvalidOperation {
            self.faulted = true;
        } else {
            self.execute_instruction(instruction);

            self.handle_timers();

            self.tick += 1;

            if self.debug_mode {
                self.debug();
            }
        }
    }

    pub fn get_screen(&mut self) -> &Screen {
        self.display.get_screen()
    }

    pub fn set_key(&mut self, key: u8, pressed: bool) {
        self.keys[key as usize] = pressed;
    }

    fn read_next_instruction(&self) -> u16 {
        let upper = self.memory[self.pc as usize] as u16;
        let lower = self.memory[(self.pc + 1) as usize] as u16;
        upper << 8 | lower
    }

    fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Clear => self.clear(),
            Instruction::Return => self.ret(),
            Instruction::ExRoutine(_a) => self.noop(), // Not sure how to implement this
            Instruction::Jump(a) => self.jump(a),
            Instruction::Call(a) => self.call(a),
            Instruction::LoadVal(r, v) => self.load_val(r, v),
            Instruction::SkipIfEqual(r, v) => self.skip_equal(r, v),
            Instruction::SkipIfNotEqual(r, v) => self.skip_not_equal(r, v),
            Instruction::SkipIfRegEqual(r1, r2) => self.skip_reg_equal(r1, r2),
            Instruction::AddVal(r, v) => self.add_val(r, v),
            Instruction::LoadReg(r1, r2) => self.load_reg(r1, r2),
            Instruction::Or(r1, r2) => self.or(r1, r2),
            Instruction::And(r1, r2) => self.and(r1, r2),
            Instruction::Xor(r1, r2) => self.xor(r1, r2),
            Instruction::AddReg(r1, r2) => self.add_reg(r1, r2),
            Instruction::SubReg(r1, r2) => self.sub_reg(r1, r2),
            Instruction::ShiftRight(r) => self.shift_right(r),
            Instruction::ShiftLeft(r) => self.shift_left(r),
            Instruction::SetIndexRegister(a) => self.set_index(a),
            Instruction::Random(r, v) => self.rand(r, v),
            Instruction::Draw(r1, r2, v) => self.draw(r1, r2, v),
            Instruction::SkipIfKey(r) => self.skip_key(r),
            Instruction::SkipIfNotKey(r) => self.skip_not_key(r),
            Instruction::AddIndex(r) => self.add_index(r),
            Instruction::LoadDigit(r) => self.load_digit(r),
            Instruction::LoadBCD(r) => self.load_bcd(r),
            Instruction::StoreIndex(r) => self.store_index(r),
            Instruction::ReadIndex(r) => self.read_index(r),
            Instruction::InvalidOperation => {}
        };
    }

    fn handle_timers(&mut self) {
        if self.tick % self.timer_tick != 0 {
            return;
        }

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

    fn push_stack(&mut self, value: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = value;
    }

    fn pop_stack(&mut self) -> u16 {
        let val = self.stack[self.sp as usize];
        self.sp -= 1;
        val
    }

    fn noop(&mut self) {
        self.pc += INSTRUCTION_SIZE;
    }

    fn load_val(&mut self, register: Register, value: Value) {
        self.set_register(register, value);
        self.pc += INSTRUCTION_SIZE;
    }

    // TODO: It is not clear whether this should overflow. Docs make it clear
    // that it should not affect VF, but they don't specify what should happen
    // during overflow. For now I am going to let it wrap.
    fn add_val(&mut self, register: Register, value: Value) {
        let current_value = self.read_register(register);
        self.set_register(register, current_value.wrapping_add(value));

        self.pc += INSTRUCTION_SIZE;
    }

    fn add_reg(&mut self, register1: Register, register2: Register) {
        let reg1_val = self.read_register(register1);
        let reg2_val = self.read_register(register2);

        let new_val: u8;
        let carry_val: u8;
        if let Some(result) = reg1_val.checked_add(reg2_val) {
            new_val = result;
            carry_val = 0;
        } else {
            // Overflow!
            // Take lower 8 bits of full addition result
            new_val = (reg1_val as u16 + reg2_val as u16) as u8;
            carry_val = 1;
        }

        self.set_register(register1, new_val);
        self.set_register(0xF, carry_val);

        self.pc += INSTRUCTION_SIZE;
    }

    fn sub_reg(&mut self, register1: Register, register2: Register) {
        let reg1_val = self.read_register(register1);
        let reg2_val = self.read_register(register2);

        let new_val: u8;
        let borrow_val: u8;
        if let Some(result) = reg1_val.checked_sub(reg2_val) {
            new_val = result;
            borrow_val = 1;
        } else {
            new_val = (reg1_val as i8 - reg2_val as i8) as u8;
            borrow_val = 0;
        }

        self.set_register(register1, new_val);
        self.set_register(0xF, borrow_val);

        self.pc += INSTRUCTION_SIZE;
    }

    fn shift_right(&mut self, register: Register) {
        let reg_val = self.read_register(register);
        self.set_register(0xF, 0b00000001 & reg_val);
        self.set_register(register, reg_val >> 1);

        self.pc += INSTRUCTION_SIZE;
    }

    fn shift_left(&mut self, register: Register) {
        let reg_val = self.read_register(register);
        self.set_register(0xF, 0b10000000 & reg_val);
        self.set_register(register, reg_val << 1);

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
        let current_pc = self.pc;
        self.push_stack(current_pc);
        self.set_program_counter(addr);
    }

    fn ret(&mut self) {
        let previous_pc = self.pop_stack();
        self.set_program_counter(previous_pc);
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

    fn skip_reg_equal(&mut self, register1: Register, register2: Register) {
        let reg_val1 = self.read_register(register1);
        let reg_val2 = self.read_register(register2);
        let pc_skip = if reg_val1 == reg_val2 {
            INSTRUCTION_SIZE * 2
        } else {
            INSTRUCTION_SIZE
        };
        self.pc += pc_skip;
    }

    fn load_reg(&mut self, register1: Register, register2: Register) {
        let reg2_val = self.read_register(register2);
        self.set_register(register1, reg2_val);
        self.pc += INSTRUCTION_SIZE;
    }

    fn or(&mut self, register1: Register, register2: Register) {
        let reg1_val = self.read_register(register1);
        let reg2_val = self.read_register(register2);
        self.set_register(register1, reg1_val | reg2_val);

        self.pc += INSTRUCTION_SIZE;
    }

    fn and(&mut self, register1: Register, register2: Register) {
        let reg1_val = self.read_register(register1);
        let reg2_val = self.read_register(register2);
        self.set_register(register1, reg1_val & reg2_val);

        self.pc += INSTRUCTION_SIZE;
    }

    fn xor(&mut self, register1: Register, register2: Register) {
        let reg1_val = self.read_register(register1);
        let reg2_val = self.read_register(register2);
        self.set_register(register1, reg1_val ^ reg2_val);

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
        for i in 0..value {
            let sprite_index = (self.index + i as u16) as usize;
            sprite.push(self.memory[sprite_index]);
        }

        let flipped = self.display.draw_sprite(&sprite, x as usize, y as usize);
        self.set_register(0xF, flipped as u8);

        self.draw_flag = true;

        self.pc += INSTRUCTION_SIZE;
    }

    fn clear(&mut self) {
        self.display.clear();

        self.pc += INSTRUCTION_SIZE;
    }

    fn skip_key(&mut self, register: Register) {
        let reg_val = self.read_register(register);
        let pc_skip = if self.keys[reg_val as usize] {
            INSTRUCTION_SIZE * 2
        } else {
            INSTRUCTION_SIZE
        };
        self.pc += pc_skip;
    }

    fn skip_not_key(&mut self, register: Register) {
        let reg_val = self.read_register(register);
        let pc_skip = if self.keys[reg_val as usize] {
            INSTRUCTION_SIZE
        } else {
            INSTRUCTION_SIZE * 2
        };
        self.pc += pc_skip;
    }

    fn add_index(&mut self, register: Register) {
        self.index += self.read_register(register) as u16;

        self.pc += INSTRUCTION_SIZE;
    }

    fn load_digit(&mut self, register: Register) {
        // Each digit sprite occupies five bytes of space starting at 0x00
        let sprite_location = 0x00 + (self.read_register(register) as u16 * 5);
        self.index = sprite_location as u16;

        self.pc += INSTRUCTION_SIZE;
    }

    fn load_bcd(&mut self, register: Register) {
        let reg_val = self.read_register(register);

        self.memory[self.index as usize] = reg_val / 100;
        self.memory[(self.index + 1) as usize] = (reg_val / 10) % 10;
        self.memory[(self.index + 2) as usize] = (reg_val % 100) % 10;

        self.pc += INSTRUCTION_SIZE;
    }

    fn store_index(&mut self, register: Register) {
        for i in 0..(register + 1) {
            let index = (self.index + i as u16) as usize;
            self.memory[index] = self.read_register(i);
        }

        self.pc += INSTRUCTION_SIZE;
    }

    fn read_index(&mut self, register: Register) {
        for i in 0..(register + 1) {
            let index = (self.index + i as u16) as usize;
            let new_val = self.memory[index];
            self.set_register(i, new_val);
        }

        self.pc += INSTRUCTION_SIZE;
    }

    fn debug(&self) {
        let reg = self.registers;
        println!(
            "V0:{} V1:{} V2:{} V3:{} V4:{} V5:{} V6:{} V7:{}",
            reg[0], reg[1], reg[2], reg[3], reg[4], reg[5], reg[6], reg[7]
        );
        println!(
            "V8:{} V9:{} VA:{} VB:{} VC:{} VD:{} VE:{} VF:{}",
            reg[8], reg[9], reg[10], reg[11], reg[12], reg[13], reg[14], reg[15]
        );
        println!(
            "I: 0x{:03X} SP: 0x{:04X} DELAY: {} SOUND: {} TICK: {}",
            self.index, self.sp, self.del_timer, self.sound_timer, self.tick
        );
        println!("MEM[I]: 0x{:02X}", self.memory[self.index as usize]);
        let keys = self.keys;
        println!(
            "K0:{} K1:{} K2:{} K3:{} K4:{} K5:{} K6:{} K7:{}",
            keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], keys[7]
        );
        println!(
            "K8:{} K9:{} KA:{} KB:{} KC:{} KD:{} KE:{} KF:{}",
            keys[8], keys[9], keys[10], keys[11], keys[12], keys[13], keys[14], keys[15]
        );
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_cpu() -> Cpu {
        return Cpu::new(Vec::new(), 360, false);
    }

    #[test]
    fn jump() {
        let mut cpu = get_cpu();
        cpu.pc = 0x0000;
        cpu.jump(0x1337);
        assert_eq!(0x1337, cpu.pc);
    }

    #[test]
    fn call_and_return() {
        let mut cpu = get_cpu();
        cpu.pc = 0x1337;
        cpu.call(0x6666);
        assert_eq!(1, cpu.sp);
        assert_eq!(0x1337, cpu.stack[cpu.sp as usize]);
        assert_eq!(0x6666, cpu.pc);
        cpu.ret();
        assert_eq!(0, cpu.sp);
        assert_eq!(0x1337, cpu.pc);
    }

    #[test]
    fn load_val() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.load_val(0x0, 0x42);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0x42, cpu.registers[0]);
    }

    #[test]
    fn skip_equal_value_equals_reg() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x42;
        cpu.skip_equal(0x0, 0x42);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 2), cpu.pc);
    }
    
    #[test]
    fn skip_equal_value_not_equals_reg() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x00;
        cpu.skip_equal(0x0, 0x42);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
    }

    #[test]
    fn skip_not_equal_value_equals_reg() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x42;
        cpu.skip_not_equal(0x0, 0x42);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
    }
    
    #[test]
    fn skip_not_equal_value_not_equals_reg() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x00;
        cpu.skip_not_equal(0x0, 0x42);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 2), cpu.pc);
    }

    #[test]
    fn skip_reg_equal_registers_equal() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x42;
        cpu.registers[1] = 0x42;
        cpu.skip_reg_equal(0x0, 0x1);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 2), cpu.pc);
    }

    #[test]
    fn skip_reg_equal_registers_not_equal() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x42;
        cpu.registers[1] = 0x00;
        cpu.skip_reg_equal(0x0, 0x1);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
    }

    #[test]
    fn add_val() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x42;
        cpu.add_val(0x0, 0x1);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0x43, cpu.registers[0]);
    }

    #[test]
    fn load_reg() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x00;
        cpu.registers[1] = 0x42;
        cpu.load_reg(0x0, 0x1);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0x42, cpu.registers[0]);
    }

    #[test]
    fn or() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;

        cpu.registers[0] = 0x00;
        cpu.registers[1] = 0x00;
        
        cpu.registers[2] = 0x00;
        cpu.registers[3] = 0x01;

        cpu.registers[4] = 0x01;
        cpu.registers[5] = 0x00;

        cpu.registers[6] = 0x01;
        cpu.registers[7] = 0x01;

        cpu.or(0x0, 0x1);
        assert_eq!(0x00, cpu.registers[0]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 1), cpu.pc);

        cpu.or(0x2, 0x3);
        assert_eq!(0x01, cpu.registers[2]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 2), cpu.pc);

        cpu.or(0x4, 0x5);
        assert_eq!(0x01, cpu.registers[4]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 3), cpu.pc);

        cpu.or(0x6, 0x7);
        assert_eq!(0x01, cpu.registers[6]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 4), cpu.pc);
    }

    #[test]
    fn and() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;

        cpu.registers[0] = 0x00;
        cpu.registers[1] = 0x00;
        
        cpu.registers[2] = 0x00;
        cpu.registers[3] = 0x01;

        cpu.registers[4] = 0x01;
        cpu.registers[5] = 0x00;

        cpu.registers[6] = 0x01;
        cpu.registers[7] = 0x01;

        cpu.and(0x0, 0x1);
        assert_eq!(0x00, cpu.registers[0]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 1), cpu.pc);

        cpu.and(0x2, 0x3);
        assert_eq!(0x00, cpu.registers[2]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 2), cpu.pc);

        cpu.and(0x4, 0x5);
        assert_eq!(0x00, cpu.registers[4]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 3), cpu.pc);

        cpu.and(0x6, 0x7);
        assert_eq!(0x01, cpu.registers[6]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 4), cpu.pc);
    }

    #[test]
    fn xor() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;

        cpu.registers[0] = 0x00;
        cpu.registers[1] = 0x00;
        
        cpu.registers[2] = 0x00;
        cpu.registers[3] = 0x01;

        cpu.registers[4] = 0x01;
        cpu.registers[5] = 0x00;

        cpu.registers[6] = 0x01;
        cpu.registers[7] = 0x01;

        cpu.xor(0x0, 0x1);
        assert_eq!(0x00, cpu.registers[0]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 1), cpu.pc);

        cpu.xor(0x2, 0x3);
        assert_eq!(0x01, cpu.registers[2]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 2), cpu.pc);

        cpu.xor(0x4, 0x5);
        assert_eq!(0x01, cpu.registers[4]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 3), cpu.pc);

        cpu.xor(0x6, 0x7);
        assert_eq!(0x00, cpu.registers[6]);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 4), cpu.pc);
    }

    #[test]
    fn add_reg() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x42;
        cpu.registers[1] = 0x01;
        cpu.add_reg(0x0, 0x1);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0x43, cpu.registers[0]);
    }

    #[test]
    fn sub_reg() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x42;
        cpu.registers[1] = 0x01;
        cpu.sub_reg(0x0, 0x1);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0x41, cpu.registers[0]);
    }

    #[test]
    fn shift_right() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0b10110101;
        cpu.shift_right(0x0);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0b01011010, cpu.registers[0]);
    }

    #[test]
    fn shift_left() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0b10110101;
        cpu.shift_left(0x0);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0b01101010, cpu.registers[0]);
    }

    #[test]
    fn set_index() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.index = 0xABCD;
        cpu.set_index(0xEFEF);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0xEFEF, cpu.index);
    }

    #[test]
    fn rand() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x42;
        cpu.rand(0x0, 0x11);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        // not asserting value difference due to lack of determinism
    }

    #[test]
    fn skip_key_pressed() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x5;
        cpu.keys[5] = true;
        cpu.skip_key(0x0);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 2), cpu.pc);
    }

    #[test]
    fn skip_key_not_pressed() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x5;
        cpu.keys[5] = false;
        cpu.skip_key(0x0);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
    }

    #[test]
    fn skip_not_key_pressed() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x5;
        cpu.keys[5] = true;
        cpu.skip_not_key(0x0);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
    }

    #[test]
    fn skip_not_key_not_pressed() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x5;
        cpu.keys[5] = false;
        cpu.skip_not_key(0x0);
        assert_eq!(initial_pc + (INSTRUCTION_SIZE * 2), cpu.pc);
    }

    #[test]
    fn add_index() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x42;
        cpu.index = 0x1101;
        cpu.add_index(0x0);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0x1143, cpu.index);
    }

    #[test]
    fn load_digit() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0xA;
        cpu.load_digit(0x0);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0x0000 + (0xA * 5), cpu.index);
    }

    #[test]
    fn load_bcd() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 209;
        cpu.load_bcd(0x0);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(2, cpu.memory[cpu.index as usize]);
        assert_eq!(0, cpu.memory[(cpu.index + 1) as usize]);
        assert_eq!(9, cpu.memory[(cpu.index + 2) as usize]);
    }

    #[test]
    fn store_index() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.registers[0] = 0x11;
        cpu.registers[1] = 0x22;
        cpu.registers[2] = 0x33;
        cpu.store_index(0x2);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0x11, cpu.memory[cpu.index as usize]);
        assert_eq!(0x22, cpu.memory[(cpu.index + 1) as usize]);
        assert_eq!(0x33, cpu.memory[(cpu.index + 2) as usize])
    }

    #[test]
    fn read_index() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        cpu.memory[cpu.index as usize] = 0x44;
        cpu.memory[(cpu.index + 1) as usize] = 0x55;
        cpu.memory[(cpu.index + 2) as usize] = 0x66;
        cpu.read_index(0x2);
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        assert_eq!(0x44, cpu.registers[0]);
        assert_eq!(0x55, cpu.registers[1]);
        assert_eq!(0x66, cpu.registers[2]);
    }

    #[test]
    fn clear() {
        let mut cpu = get_cpu();
        let initial_pc = cpu.pc;
        let mut sprite = Vec::new();
        sprite.push(0b11111111);
        cpu.display.draw_sprite(&sprite, 0, 0);
        let screen = cpu.display.get_screen();
        assert_eq!(true, screen[0][0]);
        assert_eq!(true, screen[0][1]);
        assert_eq!(true, screen[0][2]);
        assert_eq!(true, screen[0][3]);
        assert_eq!(true, screen[0][4]);
        assert_eq!(true, screen[0][5]);
        assert_eq!(true, screen[0][6]);
        assert_eq!(true, screen[0][7]);
        cpu.clear();
        assert_eq!(initial_pc + INSTRUCTION_SIZE, cpu.pc);
        let screen = cpu.display.get_screen();
        assert_eq!(false, screen[0][0]);
        assert_eq!(false, screen[0][1]);
        assert_eq!(false, screen[0][2]);
        assert_eq!(false, screen[0][3]);
        assert_eq!(false, screen[0][4]);
        assert_eq!(false, screen[0][5]);
        assert_eq!(false, screen[0][6]);
        assert_eq!(false, screen[0][7]);
    }
}
