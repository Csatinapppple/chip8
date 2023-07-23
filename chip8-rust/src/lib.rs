pub mod chip8;
//borrowed from https://github.com/starrhorne/chip8-rust/blob/master/src/processor_test.rs
const START_PC: usize = 0xF00;
const NEXT_PC: usize = START_PC + 2;
const SKIPPED_PC: usize = START_PC + (2 * 2);
fn build_processor() -> chip8::CPU {
    let mut processor = chip8::CPU::init();
    processor.pc = START_PC;
    processor.v = [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7];
    processor
}
#[cfg(test)]
mod tests{
	use super::*;
	#[test]
	fn test_00E0() {
		let mut c8 = chip8::CPU::init();
		c8.screen = [[1;64];32];
		c8.execute(0x00E0);
		for yy in 0..32{
		  for xx in 0..64{
				assert_eq!(c8.screen[yy][xx],0);        
		  }
		}
	}
    
    #[test]
    fn test_op_00ee() {
        let mut processor = chip8::CPU::init();
        processor.sp = 5;
        processor.stack[4] = 0x6666;
        processor.execute(0x00ee);
        assert_eq!(processor.sp, 4);
        assert_eq!(processor.pc, 0x6666);
    }

	#[test]
	fn test_2nnn() {
		let mut processor = build_processor();
		processor.execute(0x2888);
		assert_eq!(processor.pc, 0x0888);
		assert_eq!(processor.sp, 1);
		assert_eq!(processor.stack[0], NEXT_PC);
	}
}

