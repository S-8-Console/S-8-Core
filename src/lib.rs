pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 64;

const RAM_SIZE: usize = 1028; // 1KB
const S_REG_AMOUNT: usize = 16;
const KEY_AMOUNT: usize = 6;

const SPRITE_ROM_START_ADDR: usize = 512;

/* Our emulator struct for abstraction */
pub struct Emulator {
    ram: [u8; RAM_SIZE],
    s_regs: [u8; S_REG_AMOUNT],
    pc: u16,
    dt: u16,
    i_reg: u16,
    screen: [[u8; SCREEN_HEIGHT]; SCREEN_WIDTH],
    keys: [bool; 6],
}

impl Emulator {
    pub fn new() -> Self
    {
        return Self {
            ram: [0; RAM_SIZE],
            s_regs: [0; S_REG_AMOUNT],
            pc: 0,
            dt: 0,
            i_reg: 0,
            screen: [[0; SCREEN_HEIGHT]; SCREEN_WIDTH],
            keys: [false; KEY_AMOUNT],
        }
    }

    pub fn load(&mut self, data: &[u8], start: usize, end: usize) {
        self.ram[start..end].copy_from_slice(data);
    }


    pub fn tick(&mut self)
    {
        let op: u16 = self.fetch_op();
        self.pc+=2;

        self.execute(op);
    }

    pub fn key_down(&mut self, keycode: usize)
    {
        self.keys[keycode] = true;
    }

    pub fn key_up(&mut self, keycode: usize)
    {
        self.keys[keycode] = false;
    }

    pub fn get_screen(&mut self) -> [[u8; SCREEN_HEIGHT]; SCREEN_WIDTH]
    {
        return self.screen;
    }
    pub fn set_pc(&mut self, num: u16){
        self.pc = num;
    }

    pub fn tick_dt(&mut self)
    {
        self.dt = self.dt.overflowing_add(1).0;
    }

    fn fetch_op(&mut self) -> u16
    {
        //Get all of the game code in ram
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;

        let op = (higher_byte << 8) | lower_byte;

        return op;
    }



    fn execute(&mut self, op: u16) {
        //Get every single digit of the opcode
        let digit1 = ((op & 0xF000) >> 12) as usize;
        let digit2 = ((op & 0x0F00) >> 8) as usize;
        let digit3 = ((op & 0x00F0) >> 4) as usize;
        let digit4 = (op & 0x000F) as usize;


        match (digit1, digit2, digit3, digit4){
            //nop
            (0,0, 0, 0) => {
                return;
            },
            //clear the screen
            (0, 0, 0, 0xC) => {
                self.screen = [[0; SCREEN_HEIGHT]; SCREEN_WIDTH];
            },
            //jump to NNN
            (0x1, _, _, _) => {
                let jump_number = op & 0xFFF;
                self.pc = jump_number;
            },
            //SX = NN
            (0x2, _, _, _) => {
                let nn = (op & 0xFF) as u8;
                self.s_regs[digit2] = nn;
            },
            //SX += NN
            (0x3, _, _, _) => {
                let nn = (op & 0xFF) as u8;
                self.s_regs[digit2] = self.s_regs[digit2].overflowing_add(nn).0;
            },
            //SX -= NN
            (0x4, _, _, _) => {
                let nn = (op & 0xFF) as u8;
                self.s_regs[digit2] = self.s_regs[digit2].overflowing_sub(nn).0;
            },
            //SX = SY
            (0x5, 0, _, _) => {
                self.s_regs[digit3] = self.s_regs[digit4]
            },
            //SX += SY
            (0x5, 1, _, _) => {
                self.s_regs[digit3] = self.s_regs[digit3].overflowing_add(self.s_regs[digit4]).0;
            },
            //SX -= SY
            (0x5, 2, _, _) => {
                self.s_regs[digit3] = self.s_regs[digit3].overflowing_sub(self.s_regs[digit4]).0;
            },
            //SX *= SY
            (0x5, 3, _, _) => {
                self.s_regs[digit3] = self.s_regs[digit3].overflowing_mul(self.s_regs[digit4]).0;
            },
            //SX /= SY
            (0x5, 4, _, _) => {
                self.s_regs[digit3] = self.s_regs[digit3].overflowing_div(self.s_regs[digit4]).0;
            },
            //skip if SX == SY
            (0x6, 0, _, _) => {
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                if x == y {
                    self.pc += 2
                }
            },
            //Skip if SX != SY
            (0x6, 1, _, _) => {
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                if x != y {
                    self.pc += 2;
                }
            },
            //SX = random() byte
            (0x7, _, _, _) => {
                let nn = op & 0xFF;
                self.s_regs[digit2] = nn as u8;
            },
            //skip if key index at SX is pressed
            (0xA, 0xA, 0x0, _) => {
                let x = self.s_regs[digit4];

                if x > self.keys.len() as u8 {
                    panic!("Register value is higher than the length of the amount of keys.");
                }
                if self.keys[x as usize] {
                    self.pc += 2;
                }
            },
            //skip if key index at SX is not pressed
            (0xA, 0xA, 0x1, _) => {
                let x = self.s_regs[digit4];

                if x > self.keys.len() as u8 {
                    panic!("Register value is higher than the length of the amount of keys.");
                }
                if !self.keys[x as usize] {
                    self.pc += 2;
                }
            },
            //DAXY (draw the a'th sprite at x y)
            (0xD, _, _, _) => {
                let a = (digit2*32)+SPRITE_ROM_START_ADDR;
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                for i in 0..4 {
                    for j in 0..8 {
                        let pos_i = (i*2) as u8;
                        let full_pixel = self.ram[a+i+(j*4)];
                        let pixel1 = (full_pixel & 0xF0) >>4;
                        let pixel2 = full_pixel & 0x0F;

                        self.screen[(pos_i.overflowing_add(x).0%SCREEN_WIDTH as u8) as usize][(y.overflowing_add(j as u8).0%SCREEN_WIDTH as u8) as usize] = pixel1;
                        self.screen[(pos_i.overflowing_add(x).0.overflowing_add(1).0%SCREEN_WIDTH as u8) as usize][(y.overflowing_add(j as u8).0%SCREEN_HEIGHT as u8) as usize] = pixel2;

                    }
                }
            },
            //uninmplemented
            (_, _, _, _) => {
                unimplemented!("Opcode: {} is not implemented!", op);
            }
        }
    }
}