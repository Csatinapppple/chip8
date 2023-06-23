use std::thread;
use std::time;
use core::time::Duration;


const FONTS: [[u8;5];16] = [//col line
	[0xf0,0x90,0x90,0x90,0xf0],//0
	[0x20,0x60,0x20,0x20,0x70],//1
	[0xF0,0x10,0xF0,0x80,0xF0],//2
	[0xF0,0x10,0xF0,0x10,0xF0],//3
	[0x90,0x90,0xF0,0x10,0x10],//4
	[0xF0,0x80,0xF0,0x10,0xF0],//5
	[0xF0,0x80,0xF0,0x90,0xF0],//6
	[0xF0,0x10,0x20,0x40,0x40],//7
	[0xF0,0x90,0xF0,0x90,0xF0],//8
	[0xF0,0x90,0xF0,0x10,0xF0],//9
	[0xF0,0x90,0xF0,0x90,0x90],//a
	[0xE0,0x90,0xE0,0x90,0xE0],//b
	[0xF0,0x80,0x80,0x80,0xF0],//c
	[0xE0,0x90,0x90,0x90,0xE0],//d
	[0xF0,0x80,0xF0,0x80,0xF0],//e
	[0xF0,0x80,0xF0,0x80,0x80],//f
];

const SIXTY_HERTZ: Duration = time::Duration::from_micros(16670);
const MEMORY_BEGIN: u16 = 0x200;

struct Chip8{
    memory: [u8;0x1000],
    registers: [u8;0x10],
    screen: [[u8;64];32],
    address_register: u16,
    index_register: u16,
    nibble: u8,
    key: u8,
    sound_timer: u8,
    delay_timer: u8,
}

/*put_sprite(x_start,y_start,)
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

fn jmp(c8: &mut Chip8, index: u16){
    c8.index_register = index;
}

fn put_sprite (c8: &Chip8, sprite: &[u8], x_start: u8, y_start: u8){
    if x_start > 64 - 8 && usize::from(y_start) > 32 - sprite.len(){
    		println!("put_sprite(..,x_start = {},y_start = {}) ERROR X AND Y AXIS OUT OF BOUNDS",
    						 x_start,y_start);
    		return;
    }else if x_start > 64 - 8 {
        println!("put_sprite(..,x_start = {},y_start = {}) ERROR X AXIS OUT OF BOUNDS",
        				 x_start,y_start);
    		return;
    }else if usize::from(y_start) > 32 - sprite.len() {
        println!("put_sprite(..,x_start = {},y_start = {}) ERROR Y AXIS OUT OF BOUNDS",
        				 x_start,y_start);
        return;
    }
	//i + 7
    //to extract the last bit, bit-shift 7 times to the right
    //to extract the first bit, AND with 0b00000001 
    //(0b10011001 >> 7-n) & 0x01
    let mut i = 0usize;
    for y in y_start..y_start + sprite.len(){
        for x in x_start..x_start+8{
            c8.screen[y][x] = (sprite[y - y_start] >> 7-i) & 0x01; 
            println!("{} {}",i,(sprite[y - y_start] >> 7-i) & 0x01);
            i+=1;
        }
        i = 0;
    }
}

fn main() {
    
    let mut c8 = Chip8 {
        memory: [0;0x1000],
        registers: [0;0x10],
        screen: [[0;64];32],
        address_register: 0,
        index_register: 0,
        nibble: 0,
        key: 0,
	    sound_timer: 60,
	    delay_timer: 60,
    };
    
    loop{

        
        for y in 0..32{
            for x in 0..64{
                if c8.screen[y][x] != 0 {print!("{}",c8.screen[y][x])}
                else {print!(" ")}
            }
            println!("");
        }
        thread::sleep(time::Duration::from_secs(10));
        thread::sleep(SIXTY_HERTZ);
        c8.delay_timer-=1;
        c8.sound_timer-=1;
        if c8.delay_timer == 0 {c8.delay_timer = 60}
        if c8.sound_timer == 0 {c8.sound_timer = 60}
        println!(
            "delay timer = {} sound_timer = {}",
            c8.delay_timer, c8.sound_timer    
        );
    
    }
    
    /* 
    for y in 0..16 {
    	for x in 0..5{
    		//println!("x = {}, y = {}",x,y);
    		println!("{:#010b}",FONTS[y][x]);
    	}
    	println!("{}",y);
    }
    println!("Hello, world!");
    */
}
