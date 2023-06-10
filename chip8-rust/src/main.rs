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

struct Chip8{
    memory: [u8;0x1000],
    registers: [u8;0x10],
    screen: [[u8;32];64],
    address_register: u16,
    index_register: u16,
    key: u8,
}

fn main() {
    
    let c8 = Chip8 {
        memory: [0;0x1000],
        registers: [0;0x10],
        screen: [[0;32];64],
        address_register: 0,
        index_register: 0,
        key: 0,
    };
    
    for y in 0..16 {
    	for x in 0..5{
    		//println!("x = {}, y = {}",x,y);
    		println!("{:#010b}",FONTS[y][x]);
    	}
    	println!("{}",y);
    }

    println!("Hello, world!");
}
