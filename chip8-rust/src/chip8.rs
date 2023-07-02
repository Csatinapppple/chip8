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
    pub memory: [u8; 0x1000],
    pub registers: [u8; 16],
    pub _I: u16,
    pub screen: [[u8;64];32],
    pub sound_timer: u8,
    pub delay_timer: u8,
    pub program_counter: u16,
    pub stack: [u16; 16],
    pub stack_pointer: u8,
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
    
    //1NNN (jump)
    pub fn jump(&mut self,pos: u16){
        self.program_counter = pos;
    }

    //00EE return from subroutine
    pub fn ret_subroutine(&mut self){
        self.program_counter = self.stack[usize::from(self.stack_pointer)];
        self.stack_pointer -= 1;
    }

    //2NNN return from subroutine at address
    pub fn ret_from_sub_at_addr(&mut self,pos: u16){
        self.stack_pointer +=1;
        self.stack[usize::from(self.stack_pointer)] = self.program_counter;
        self.program_counter = pos;
    }

    pub fn decode(&mut self) {
        let x = self.fetch();
        self.program_counter += 2;
        match x & 0xf000 {
            0x0000 => match x {
                0x00E0 => self.clear_screen(),
                0x00EE => self.ret_subroutine(),
            },
            0x1000 => self.jump(x & 0x0FFF),
            0x2000 => self.ret_from_sub_at_addr(x & 0x0FFF)
        }
    }

    pub fn fetch(&self) -> u16 {
        self.memory[usize::from(self.program_counter+1)] as u16 |
        (self.memory[usize::from(self.program_counter)] as u16) <<8
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
            memory: [0; 0x1000],
            registers: [0; 16],
            _I: 0,
            screen: [[0; 64] ; 32],
            sound_timer: 0,
            delay_timer: 0,
            program_counter: 0x200,
            stack: [0; 16],
            stack_pointer: 0,
        };

        for y in 0..16{
            for x in  0..5{//common font storage is 0x050 to 0x01ff
                ini.memory[ y * 5 + x + 0x050 ]=FONTS[y][x];
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
            self.memory[i+0x200] = buffer[i];
            println!("byte = {}",buffer[i]);
        }

        Ok(())
    }
}

