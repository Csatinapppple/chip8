const mem_start = 0x200;

struct Chip8{
    memory: u8[0x1000];
    registers: u8[16];
    graphics: u8[64][32];
    memory_address: u16;
}

impl Chip8{

    init(&self){
        memory = 0;
        registers = 0;
        graphics = 0;
        memory_address = 0;
    }

}
