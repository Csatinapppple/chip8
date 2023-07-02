use {
    core::time::Duration, 
    std::{
        thread,
        time,
    },
};

mod chip8;

fn main() {
    //initiate cpu
    let mut c8 = chip8::CPU::init();
    //load rom into cpu
    c8.load_file("rom/ibm_logo.bin");
    //render and call emulate cycle
    loop {
        println!(
            "fetch = {:#06x}",
            c8.fetch()
        );
        c8.program_counter += 2;
        for y in 0..32 {
            for x in 0..64 {
                match c8.screen[y][x] {
                    0 => print!(" "),
                    _ => print!("{}", c8.screen[y][x]),
                }
            }
            println!("");
        }
        thread::sleep(time::Duration::from_secs(10));
    }
}
