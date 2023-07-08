extern crate rand;

use {
    core::time::Duration, 
    std::{
        thread,
        time,
        io,
        io::{
            Read,
            BufReader,
        },
        fs::File,
    },
    rand::Rng,
};

pub const FONTS: [[u8; 5]; 16] = [
    //col line
    [0xf0, 0x90, 0x90, 0x90, 0xf0], //0
    [0x20, 0x60, 0x20, 0x20, 0x70], //1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], //2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], //3
    [0x90, 0x90, 0xF0, 0x10, 0x10], //4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], //5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], //6
    [0xF0, 0x10, 0x20, 0x40, 0x40], //7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], //8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], //9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], //a
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], //b
    [0xF0, 0x80, 0x80, 0x80, 0xF0], //c
    [0xE0, 0x90, 0x90, 0x90, 0xE0], //d
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], //e
    [0xF0, 0x80, 0xF0, 0x80, 0x80], //f
];
//&FONTS[0][..]

//currently frame limiting just sleeps for 60hz
const SIXTY_HERTZ: Duration = time::Duration::from_micros(16670);

pub struct CPU {
    pub mem: [u8; 0x1000],
    pub v: [u8; 16], //registers
    pub _I: usize,
    pub screen: [[u8;64];32],
    pub st: u8,//sound timer
    pub dt: u8,//delay timer
    pub pc: usize,//program counter
    pub stack: [usize; 16],
    pub sp: usize,//stack pointer
}



impl CPU{
    
    //00E0 (clear screen)
    pub fn clear_screen(&mut self){
        for y in 0..32{
            for x in 0..64{
                self.screen[y][x] = 0;
            }
        }
    }
    
    //00EE return from subroutine
    pub fn ret_subroutine(&mut self){
        self.pc = self.stack[self.sp];
        self.sp -= 1;
    }
 
    //2NNN return from subroutine at address
    pub fn ret_from_sub_at_addr(&mut self,nnn: usize){
        self.sp +=1;
        self.stack[self.sp] = self.pc;
        self.pc = nnn;
    }

    //3XNN & 4XNN skips if the eq flag is true and cond is met so is the inverse
    pub fn skip_if(&mut self,x: usize, nn: u8, eq: bool){
        let cond = self.v[x] == nn;
        if cond && eq{
            self.pc += 2;
        }else if !cond && !eq {
            self.pc += 2;
        }
    }
    
    //5XY0 & 9XY0 same as skip_if but for registers
    pub fn skip_if_reg(&mut self,x: usize,y: usize,eq: bool){
        let cond = self.v[x] == self.v[y];
        if cond && eq{
            self.pc += 2;
        }else if !cond && !eq{
            self.pc += 2;
        }
    }
    
    //8XY4 add vX with vY and set vF to 1 in overflow
    pub fn add_x_y(&mut self,x:usize,y:usize){
        let cmp = self.v[x];
        self.v[x] += self.v[y];
        if self.v[x] < cmp {
            self.v[0xF] = 1;
        }
    }

    /*
        8XY5 & 8XY7 subtracts vX with vY or vY with vX and sets it to the vX
        in case where the first operand is bigger than the second operand ie.
        (n+1) - n it sets vF to 1 and the inverse sets it to 0
     */ 
    pub fn sub_x_y(&mut self, x:usize,y:usize,inv:bool){
        if inv {
            if self.v[y] >= self.v[x] {
                self.v[0xF] = 1;
            } else{
                self.v[0xF] = 0;
            }
            self.v[x] = self.v[y] - self.v[x];
        }
        else {
            if self.v[x] >= self.v[y] {
                self.v[0xF] = 1;
            } else{
                self.v[0xF] = 0;
            }
            self.v[x] -= self.v[y];
        }
    }

    /*
        original COSMAC VIP
        8XY6 & 8XYE shift 1 bit to the right if right flag is set to true
        and set vF to 1 if the bit that was shifted out was 1 and vice versa
        works for the last bit shifted to the right and the first bit to the
        left
    */
    pub fn shift(&mut self, x:usize, y:usize, right:bool){
        self.v[x] = self.v[y];
        match right {
            true => {
                if self.v[x] & 0b00000001 == 1{
                    self.v[0xf] = 1;
                }
                else {
                    self.v[0xf] = 0;
                }
                self.v[x] >> 1;
            },
            false => {
                if self.v[x] & 0b10000000 == 1{
                    self.v[0xf] = 1;
                }
                else {
                    self.v[0xf] = 0;
                }
                self.v[x] << 1;
            },
        }
    }

    //CXNN generates random number and puts the result in vX after ANDing with nn
    pub fn random(&mut self, x: usize, nn: u8){
        let random_number: u8 = rand::thread_rng().gen();
        self.v[x] = random_number & nn;
    }

    //DYXN display,draw
    pub fn draw(&mut self, y:usize,x:usize,n:usize){
        for y_screen in 0..n{
            let line: u8 = self.mem[self._I+y_screen];
            for x_screen in 0..8{
            }
        }
    }
    pub fn decode(&mut self) {
        let opcode: u16 = self.fetch();
        self.pc += 2;
        
        let nnn: usize = (opcode & 0x0FFF).into();
        let nn: u8 = (opcode & 0x00FF) as u8;
        let n: usize = (opcode & 0x000F).into();
        let x: usize = ((opcode & 0x0F00) >> 8).into();
        let y: usize = ((opcode & 0x00F0) >> 4).into();

        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.clear_screen(),
                0x00EE => self.ret_subroutine(),
                _ => unreachable!(),
            },
            0x1000 => self.pc = nnn, //1NNN set pc to NNN (jump)
            0x2000 => self.ret_from_sub_at_addr(nnn),
            0x3000 => self.skip_if(x,nn,true),//skip if equal
            0x4000 => self.skip_if(x,nn,false),//skip if different
            0x5000 => self.skip_if_reg(x,y,true),
            0x6000 => self.v[x] = nn, //6XNN set vX to NN
            0x7000 => self.v[x] += nn,//7XNN add NN to vX without setting vF
            0x8000 => match opcode & 0x000F {
                //arithmetic opcodes
                0x0000 => self.v[x]=self.v[y], //8XY0 set vX to vY
                0x0001 => self.v[x]|=self.v[y],//8XY1 OR vX with vY
                0x0002 => self.v[x]&=self.v[y],//8XY2 AND vX with vY
                0x0003 => self.v[x]^=self.v[y],//8XY3 XOR vX with vY
                0x0004 => self.add_x_y(x,y),
                0x0005 => self.sub_x_y(x,y,false),
                0x0006 => self.shift(x,y,true),
                0x0007 => self.sub_x_y(x,y,true),
                0x000E => self.shift(x,y,false),
                _ => unreachable!(),
            }
            0x9000 => self.skip_if_reg(x,y,false),
            0xA000 => self._I = nnn, //ANNN set index to NNN
            0xB000 => self.pc = nnn + usize::from(self.v[0]), //BNNN COSMAC VIP jumps to nnn + v0
            0xC000 => self.random(x,nn),
            //0xD000 => self.put_sprite()
            _ => unreachable!(),

        }
    }

    pub fn fetch(&self) -> u16 {
        self.mem[self.pc+1] as u16 |
        (self.mem[self.pc] as u16) <<8
    }



    /*put_sprite(x_start,y_start,)DXYN
                |
                v x_start_max
    |       5 6 7 8 9 0 1 2 3 64      iterates and replaces each byte
          5 0 0 0 0 0 0 0 0 0 0       with the binary of the byte
          6 0 0 0 0 0 0 0 0 0 0       y_start_max = 32 - sprite.len()
          7 0 0 0 0 0 0 0 0 0 0
          8 0 0 0 0 0 0 0 0 0 0
          9 0 0 0 0 0 0 0 0 0 0
          0 0 0 0 0 0 0 0 0 0 0
          1 0 0 0 0 0 0 0 0 0 0
         32 0 0 0 0 0 0 0 0 0 0

    */
    pub fn put_sprite(&mut self, sprite: &[u8], x_start: usize, y_start: usize) {
        if x_start > 64 - 8 && y_start > 32 - sprite.len() {
            println!(
                "put_sprite(..,x_start = {},y_start = {}) ERROR X AND Y AXIS OUT OF BOUNDS",
                x_start, y_start
            );
            return;
        } else if x_start > 64 - 8 {
            println!(
                "put_sprite(..,x_start = {},y_start = {}) ERROR X AXIS OUT OF BOUNDS",
                x_start, y_start
            );
            return;
        } else if y_start > 32 - sprite.len() {
            println!(
                "put_sprite(..,x_start = {},y_start = {}) ERROR Y AXIS OUT OF BOUNDS",
                x_start, y_start
            );
            return;
        }
        //i + 7
        //to extract the last bit, bit-shift 7 times to the right
        //to extract the first bit, AND with 0b00000001
        //(0b10011001 >> 7-n) & 0x01
        let mut i = 0usize;
        for y in y_start..y_start + sprite.len() {
            for x in x_start..x_start + 8 {
                self.screen[ y ][ x ] = (sprite[y - y_start] >> 7 - i) & 0x01;
                println!("{} {}", i, (sprite[y - y_start] >> 7 - i) & 0x01);
                i += 1;
            }
            i = 0;
        }
    }


    //Initializes variables to default settings
    pub fn init() -> CPU{
        let mut ini = CPU{
            mem: [0; 0x1000],
            v: [0; 16],
            _I: 0,
            screen: [[0; 64] ; 32],
            st: 0,
            dt: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
        };

        for y in 0..16{
            for x in  0..5{//common font storage is 0x050 to 0x01ff
                ini.mem[ y * 5 + x ]=FONTS[y][x];
            }
        }
        ini
    }
    //loads file into memory
    pub fn load_file(&mut self,filepath: &str) -> io::Result<()>{
        let f = File::open(filepath)?;
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        //Read file into vector
        reader.read_to_end(&mut buffer)?;

        //Read
        for i in 0..buffer.len() {
            self.mem[i+0x200] = buffer[i];
            println!("byte = {}",buffer[i]);
        }

        Ok(())
    }
}

