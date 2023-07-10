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
};
use crate::chip8::rand::Rng;

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
pub const SIXTY_HERTZ: Duration = time::Duration::from_micros(16670);

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
    pub keys: [bool; 16],
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
    //8XY5
    pub fn sub_x_with_y(&mut self, x:usize,y:usize){
        if self.v[x] >= self.v[y] {
            self.v[0xF] = 1;
        } else{
            self.v[0xF] = 0;
        }
        self.v[x] -= self.v[y];
    }
    //8XY7
    pub fn sub_x_with_y_x(&mut self, x:usize,y:usize){
        if self.v[y] >= self.v[x] {
            self.v[0xF] = 1;
        } else{
            self.v[0xF] = 0;
        }
        self.v[x] = self.v[y] - self.v[x];
    }

    /*
        original COSMAC VIP
        8XY6 & 8XYE shift 1 bit to the right if right flag is set to true
        and set vF to 1 if the bit that was shifted out was 1 and vice versa
        works for the last bit shifted to the right and the first bit to the
        left
    */
    //8XY6
    pub fn shift_right(&mut self, x:usize, y:usize){
        self.v[x] = self.v[y];    
        if self.v[x] & 0b00000001 == 1{
            self.v[0xf] = 1;
        }
        else {
            self.v[0xf] = 0;
        }
        self.v[x] >>= 1;
    }
    //8XYE
    pub fn shift_left(&mut self, x:usize, y:usize){
        self.v[x] = self.v[y];
        if self.v[x] & 0b10000000 == 1{
            self.v[0xf] = 1;
        }
        else {
            self.v[0xf] = 0;
        }
        self.v[x] <<= 1;
    }

    //CXNN generates random number and puts the result in vX after ANDing with nn
    pub fn random(&mut self, x: usize, nn: u8){
        let random_number: u8 = rand::thread_rng().gen();
        self.v[x] = random_number & nn;
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
            0x3000 => self.pc += if self.v[x] == nn {2} else {0},//3XNN skip if equal
            0x4000 => self.pc += if self.v[x] != nn {2} else {0},//4XNN skip if different 
            0x5000 => self.pc += if self.v[x] == self.v[y] {2} else {0},//5XY0 skip if equal
            0x6000 => self.v[x] = nn, //6XNN set vX to NN
            0x7000 => self.v[x] += nn,//7XNN add NN to vX without setting vF
            0x8000 => match opcode & 0x000F {
                //arithmetic opcodes
                0x0000 => self.v[x]=self.v[y], //8XY0 set vX to vY
                0x0001 => self.v[x]|=self.v[y],//8XY1 OR vX with vY
                0x0002 => self.v[x]&=self.v[y],//8XY2 AND vX with vY
                0x0003 => self.v[x]^=self.v[y],//8XY3 XOR vX with vY
                0x0004 => self.add_x_y(x,y),
                0x0005 => self.sub_x_with_y(x,y),
                0x0006 => self.shift_right(x,y),
                0x0007 => self.sub_x_with_y_x(x,y),
                0x000E => self.shift_left(x,y),
                _ => unreachable!(),
            }
            0x9000 => self.pc += if self.v[x] != self.v[y] {2} else {0},//9XY0 skip if different
            0xA000 => self._I = nnn, //ANNN set index to NNN
            0xB000 => self.pc = nnn + usize::from(self.v[0]), //BNNN COSMAC VIP jumps to nnn + v0
            0xC000 => self.random(x,nn),//CXNN get random number
            0xD000 => self.draw(x,y,n), //DXYN draw display sprite
            0xE000 => match opcode & 0x00FF {
                0x009E => self.pc += if self.keys[usize::from(self.v[x])]{2}else{0}, //EX9E skip if vX key is true
                0x00A1 => self.pc += if !self.keys[usize::from(self.v[x])]{2}else{0}, //EXA1 skip if vX key is false
                _ => unreachable!(),
            }
            _ => unreachable!(),

        }
    }

    pub fn fetch(&self) -> u16 {
        self.mem[self.pc+1] as u16 |
        (self.mem[self.pc] as u16) <<8
    }


    //DXYN display,draw
    pub fn draw(&mut self, x_reg:usize,y_reg:usize,n:usize){
        let mut i: usize = 0;
        let mut y_coord: usize = self.v[y_reg].into();
        let mut x_coord: usize = self.v[x_reg].into();//height,width = y_reg,x_reg
        println!("x = {} y = {} n = {}",y_coord,x_coord,n);
        for y in y_coord..y_coord+n {
            //offset of n present in y_coord+n is the amount of bytes in the sprite starting from I
            for x in x_coord..x_coord+8 {
                if x >= 64 || y >= 32 {
                    continue;
                }
                else{
                    self.screen[y][x] = (self.mem[self._I + y - y_coord] >> 7 - i) & 0x1;
                    i+=1;
                    println!(
                        "[{}][{}]",
                        y,x
                    );
                }
                
            }
            i=0;
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
            keys: [false; 16],
        };

        for y in 0..16{
            for x in  0..5{//common font storage is 0x050 to 0x01ff
                ini.mem[ y * 5 + x + 0x050 ]=FONTS[y][x];
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

