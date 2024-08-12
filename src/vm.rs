use rand::Rng;

pub const MEMORY_SIZE: usize = 65536; // 64KB of memory
pub const VRAM_START: usize = 0xF000; // Start of video memory
pub const VRAM_SIZE: usize = 1000; // 40x25 text mode display
pub const INPUT_REGISTER: usize = 0xFFF0; // Memory-mapped input register
pub const OUTPUT_REGISTER: usize = 0xFFF1; // Memory-mapped output register
pub const RANDOM_REGISTER: usize = 0xFFF2; // Memory-mapped random number generator
pub const TIMER_REGISTER: usize = 0xFFF3; // Memory-mapped timer register

// In vm.rs
pub const INPUT_UP: u8 = 10;
pub const INPUT_DOWN: u8 = 20;
pub const INPUT_LEFT: u8 = 30;
pub const INPUT_RIGHT: u8 = 40;
pub const INPUT_START: u8 = 50;
pub const INPUT_SELECT: u8 = 60;
pub const INPUT_A: u8 = 70;
pub const INPUT_B: u8 = 80;
pub const INPUT_NONE: u8 = 0;

#[derive(Clone, Debug)]
pub enum Instruction {
    Nop,
    Push(i32),
    Pop,
    Dup,
    Swap,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Not,
    Eq,
    Ne,
    Lt,
    Gt,
    Lte,
    Gte,
    Jump(usize),
    JumpIf(usize),
    Call(usize),
    Ret,
    Load(usize),
    Store(usize),
    LoadImmediate(usize, i32),
    RandomNum(i32, i32),
    Sleep(u64),
    ClearScreen,
    Halt,
}

pub struct VM {
    pub memory: [u8; MEMORY_SIZE],
    stack: Vec<i32>,
    pub program: Vec<Instruction>,
    pc: usize,
    call_stack: Vec<usize>,
    pub halted: bool,
    timer: u64,
    pub screen_dirty: bool,
    input_state: u8,
}

impl VM {
    pub fn new() -> Self {
        VM {
            memory: [0; MEMORY_SIZE],
            stack: Vec::new(),
            program: Vec::new(),
            pc: 0,
            call_stack: Vec::new(),
            halted: false,
            timer: 0,
            screen_dirty: false,
            input_state: INPUT_NONE,
        }
    }

    pub fn load_program(&mut self, program: Vec<Instruction>) {
        self.program = program;
        self.pc = 0;
        self.halted = false;
    }

    pub fn load_bios(&mut self, filename: &str) {
        let bios = std::fs::read(filename).expect("Failed to read BIOS file");
        self.memory[..bios.len()].copy_from_slice(&bios);

        // Set the program counter to the start of the BIOS
        self.pc = 0;
    }

    pub fn run_cycle(&mut self) {
        if self.halted || self.pc >= self.program.len() {
            return;
        }

        match &self.program[self.pc].clone() {
            Instruction::Nop => {}
            Instruction::Push(value) => self.stack.push(*value),
            Instruction::Pop => {
                self.stack.pop();
            }
            Instruction::Dup => {
                if let Some(&value) = self.stack.last() {
                    self.stack.push(value);
                }
            }
            Instruction::Swap => {
                if self.stack.len() >= 2 {
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 2);
                }
            }
            Instruction::Add => self.binary_op(|a, b| a + b),
            Instruction::Sub => self.binary_op(|a, b| a - b),
            Instruction::Mul => self.binary_op(|a, b| a * b),
            Instruction::Div => self.binary_op(|a, b| a / b),
            Instruction::Mod => self.binary_op(|a, b| a % b),
            Instruction::And => self.binary_op(|a, b| a & b),
            Instruction::Or => self.binary_op(|a, b| a | b),
            Instruction::Xor => self.binary_op(|a, b| a ^ b),
            Instruction::Not => {
                if let Some(value) = self.stack.pop() {
                    self.stack.push(!value);
                }
            }
            Instruction::Eq => self.compare_op(|a, b| a == b),
            Instruction::Ne => self.compare_op(|a, b| a != b),
            Instruction::Lt => self.compare_op(|a, b| a < b),
            Instruction::Gt => self.compare_op(|a, b| a > b),
            Instruction::Lte => self.compare_op(|a, b| a <= b),
            Instruction::Gte => self.compare_op(|a, b| a >= b),
            Instruction::Jump(addr) => {
                self.pc = *addr;
                return;
            }
            Instruction::JumpIf(addr) => {
                if let Some(value) = self.stack.pop() {
                    if value != 0 {
                        self.pc = *addr;
                        return;
                    }
                }
            }
            Instruction::Call(addr) => {
                self.call_stack.push(self.pc + 1);
                self.pc = *addr;
                return;
            }
            Instruction::Ret => {
                if let Some(addr) = self.call_stack.pop() {
                    self.pc = addr;
                    return;
                }
            }
            Instruction::Load(addr) => {
                let value = self.read_memory(*addr);
                self.stack.push(value as i32);
            }
            Instruction::Store(addr) => {
                if let Some(value) = self.stack.pop() {
                    self.write_memory(*addr, value as u8);
                }
            }
            Instruction::LoadImmediate(addr, value) => {
                self.write_memory(*addr, *value as u8);
            }
            Instruction::RandomNum(min, max) => {
                let num = rand::thread_rng().gen_range(*min..=*max);
                self.write_memory(RANDOM_REGISTER, num as u8);
            }
            Instruction::Sleep(ms) => {
                self.timer = *ms;
            }
            Instruction::ClearScreen => {
                for i in VRAM_START..(VRAM_START + VRAM_SIZE) {
                    self.memory[i] = 0;
                }
            }
            Instruction::Halt => {
                self.halted = true;
                return;
            }
        }

        self.pc += 1;
    }

    fn binary_op<F>(&mut self, op: F)
    where
        F: Fn(i32, i32) -> i32,
    {
        if self.stack.len() >= 2 {
            let b = self.stack.pop().unwrap();
            let a = self.stack.pop().unwrap();
            self.stack.push(op(a, b));
        }
    }

    fn compare_op<F>(&mut self, op: F)
    where
        F: Fn(i32, i32) -> bool,
    {
        if self.stack.len() >= 2 {
            let b = self.stack.pop().unwrap();
            let a = self.stack.pop().unwrap();
            self.stack.push(if op(a, b) { 1 } else { 0 });
        }
    }

    pub fn read_memory(&self, addr: usize) -> u8 {
        match addr {
            INPUT_REGISTER => self.input_state,
            RANDOM_REGISTER => self.memory[RANDOM_REGISTER],
            TIMER_REGISTER => self.timer as u8,
            _ => self.memory[addr],
        }
    }

    pub fn write_memory(&mut self, addr: usize, value: u8) {
        match addr {
            OUTPUT_REGISTER => {
                println!("Output: {}", value as char);
            }
            addr if (VRAM_START..VRAM_START + VRAM_SIZE).contains(&addr) => {
                self.memory[addr] = value;
                self.screen_dirty = true;
            }
            _ => self.memory[addr] = value,
        }
    }

    pub fn update_timer(&mut self, delta_ms: u64) {
        if self.timer > 0 {
            self.timer = self.timer.saturating_sub(delta_ms);
        }
    }

    pub fn set_input(&mut self, input: u8) {
        self.input_state = input;
    }

    pub fn check_input(&self) -> u8 {
        self.input_state
    }

    pub fn vblank_interrupt(&mut self) {
        // Simulate the vertical blanking interval

        // 1. Update timers
        // In many systems, timers are updated during VBlank
        if self.timer > 0 {
            self.timer -= 1;
        }

        // 2. Handle sound
        // If you implement sound, you might update sound registers here

        // 3. Update input
        // Some systems read input during VBlank
        // For now, we'll just ensure the input state is current
        self.memory[INPUT_REGISTER] = self.input_state;

        // 4. Trigger any VBlank-specific interrupts
        // In a more complex system, you might have interrupt vectors
        // For now, we'll just set a flag that could be checked by the program
        self.memory[0xFFF4] = 1; // Set a VBlank flag at address 0xFFF4

        // 5. Signal that it's safe to update the screen
        // This is what we're using the screen_dirty flag for
        if self.screen_dirty {
            // In a real system, you might set a flag that the program can check
            // to know it's safe to update the screen
            self.memory[0xFFF5] = 1; // Set a "safe to draw" flag at address 0xFFF5
        }

        // 6. Reset the screen_dirty flag
        // This is done here rather than in the rendering code because
        // in a real system, the VBlank period is when you know the screen
        // isn't being actively drawn to
        self.screen_dirty = false;
    }
}
