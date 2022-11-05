use rand::Rng;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 64;

const RAM_SIZE: usize = 2208;
const S_REG_AMOUNT: usize = 16;
const KEY_AMOUNT: usize = 6;

const SPRITE_ROM_START_ADDR: usize = 512;
const FONT_ROM_START_ADDR: usize = 1024;

/* Our emulator struct for abstraction */
pub struct Emulator {
    ram: [u8; RAM_SIZE],
    s_regs: [u8; S_REG_AMOUNT],
    pc: u16,
    dt: u16,
    loop_point: u16,
    f_reg: u8,
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
            loop_point: 0,
            f_reg: 0xF,
            screen: [[0; SCREEN_HEIGHT]; SCREEN_WIDTH],
            keys: [false; KEY_AMOUNT],
        }
    }


    pub fn get_loop_point(&mut self) -> u16
    {
        return self.loop_point;
    }

    pub fn load(&mut self, data: &[u8], start: usize, end: usize) {
        self.ram[start..end].copy_from_slice(data);
    }


    pub fn tick(&mut self) -> u16
    {
        let op: u16 = self.fetch_op();
        self.pc+=2;

        self.execute(op);

        return self.pc;
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

    fn draw_font(&mut self, x: u8, y: u8, z: usize)
    {
        for i in 0..4 {
            for j in 0..8 {
                let pos_i = (i*2) as u8;
                let full_pixel = self.ram[z +i+(j*4)];
                let pixel1 = (full_pixel & 0xF0) >>4;
                let pixel2 = full_pixel & 0x0F;

                /* if pixels are 0, we are not going to write them to the screen array because we want them to be transparent
                    meaning we don't want those black pixels to overlap other colors
                 */
                if pixel1 == 0xF
                {
                    self.screen[(pos_i.overflowing_add(x).0%SCREEN_WIDTH as u8) as usize][(y.overflowing_add(j as u8).0%SCREEN_WIDTH as u8) as usize] = self.f_reg;
                }
                if pixel2 == 0xF
                {
                    self.screen[(pos_i.overflowing_add(x).0.overflowing_add(1).0%SCREEN_WIDTH as u8) as usize][(y.overflowing_add(j as u8).0%SCREEN_HEIGHT as u8) as usize] = self.f_reg;
                }

            }
        }
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
            (0x5, 5, _, _) => {
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                if x == y {
                    self.pc += 2
                }
            },
            //Skip if SX != SY
            (0x5, 6, _, _) => {
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                if x != y {
                    self.pc += 2;
                }
            },
            //Skip if SX > SY
            (0x5, 7, _, _) => {
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                if x > y {
                    self.pc += 2;
                }
            },
            //Skip if SX < SY and SX + 8 > SY
            (0x5, 8, _, _) => {
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                if x < y && x.overflowing_add(8).0 > y {
                    self.pc += 2;
                }
            },
            //Skip if SX < SY+8 and SX + 8 > SY+8
            (0x5, 9, _, _) => {
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                if x < y.overflowing_add(8).0%64 && x + 8 > y.overflowing_add(8).0%64 {
                    self.pc += 2;
                }
            },
            //Skip if SX < SY+8 and SX + 8 > SY+8 or Skip if SX < SY and SX + 8 > SY
            (0x5, 0xA, _, _) => {
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                if x < y.overflowing_add(8).0%64 && x + 8 > y.overflowing_add(8).0%64 || x < y && x.overflowing_add(8).0%64 > y || x == y {
                    self.pc += 2;
                }
            },
            //600N Set font reg to N
            (0x6, 0x0, 0x0, _) => {
                let x = digit4 as u8;

                self.f_reg = x;
            },
            //600N Set font reg to SX%16
            (0x6, 0x1, 0x0, _) => {
                let x = digit4;

                self.f_reg = self.s_regs[x]%16;
            },
            //SX = random() hex number
            (0x7, 0x0, 0x0, _) => {
                let mut rng = rand::thread_rng();

                let num: u8 = rng.gen_range(0..16);
                self.s_regs[digit4] = num;
            },
            //Loop_point = SX%16
            (0x7, 0x1, 0x0, _) => {
                self.loop_point = self.s_regs[digit4] as u16%16; // % 16 because it is used as a color
            },
            //800N loop_point = N
            (0x8, 0x0, 0x0, _) => {
                let n = digit4;

                self.loop_point = n as u16;
            },
            //Skip if key index at SX is pressed
            (0x9, 0x0, 0x0, _) => {
                let x = self.s_regs[digit4];

                if x > self.keys.len() as u8 {
                    panic!("Register value is higher than the length of the amount of keys.");
                }
                if self.keys[x as usize] {
                    self.pc += 2;
                }
            },
            //Skip if key index at SX is not pressed
            (0x9, 0x1, 0x0, _) => {
                let x = self.s_regs[digit4];

                if x > self.keys.len() as u8 {
                    panic!("Register value is higher than the length of the amount of keys.");
                }
                if !self.keys[x as usize] {
                    self.pc += 2;
                }
            },
            //ANXY Skip N amount if X != Y
            (0xA, _, _, _) => {
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                if x != y {
                    self.pc = self.pc.overflowing_add((digit2 as u16*2 as u16).try_into().unwrap()).0;
                }
            },
            //DNXY (draw the n'th sprite at SX SY)
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

                        /* if pixels are 0, we are not going to write them to the screen array because we want them to be transparent
                            meaning we don't want those black pixels to overlap other colors
                         */
                        if pixel1 != 0
                        {
                            self.screen[(pos_i.overflowing_add(x).0%SCREEN_WIDTH as u8) as usize][(y.overflowing_add(j as u8).0%SCREEN_WIDTH as u8) as usize] = pixel1;
                        }
                        if pixel2 != 0
                        {
                            self.screen[(pos_i.overflowing_add(x).0.overflowing_add(1).0%SCREEN_WIDTH as u8) as usize][(y.overflowing_add(j as u8).0%SCREEN_HEIGHT as u8) as usize] = pixel2;
                        }

                    }
                }
            },
            //EZXY (draw the value of SZ at SX SY)
            (0xE, _, _, _) => {
                let z = self.s_regs[digit2];

                let z_string = format!("{}", z);
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                for (i, v) in z_string.chars().enumerate()
                {
                    let num_to_draw = v.to_digit(10).unwrap() as usize;

                    self.draw_font(x+(i*8) as u8, y, (num_to_draw*32)+FONT_ROM_START_ADDR);
                }
            },
            //FZXY (draw the z'th font at x y)
            (0xF, _, _, _) => {
                let z = (self.s_regs[digit2] as usize*32)+FONT_ROM_START_ADDR;
                let x = self.s_regs[digit3];
                let y = self.s_regs[digit4];

                self.draw_font(x, y, z);
            },
            //uninmplemented
            (_, _, _, _) => {
                unimplemented!("Opcode {:#06x} at pc {} is not implemented!", op, self.pc);
            }
        }
    }
}

