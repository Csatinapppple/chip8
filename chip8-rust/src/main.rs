use std::thread;
use std::time;
use core::time::Duration;


const FONTS: [[u8;5];16] = [//ROW COL
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
    screen: [[u8;32];64],
    address_register: u16,
    index_register: u16,
    key: u8,
    sound_timer: u8,
    delay_timer: u8,
}

fn main() {
    
    let mut c8 = Chip8 {
        memory: [0;0x1000],
        registers: [0;0x10],
        screen: [[0;32];64],
        address_register: 0,
        index_register: 0,
        key: 0,
	sound_timer: 60,
	delay_timer: 60,
    };

    loop{

    
    
    thread::sleep(SIXTY_HERTZ);
    c8.delay_timer-=1;
    c8.sound_timer-=1;
    }

    /*
    for y in 0..16 {
    	for x in 0..5{
    		//println!("x = {}, y = {}",x,y);
    		println!("{:# 10b}",FONTS[y][x]);
    	}
    	println!("{}",y);
    }
    println!("Hello, world!");
    */
}
