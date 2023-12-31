/**
* Chip-8 
* 
* Chip-8 is a simple, interpreted, programming language which was first used on some do-it-yourself computer systems 
* in the late 1970s and early 1980s. The COSMAC VIP, DREAM 6800, and ETI 660 computers are a few examples. 
* These computers typically were designed to use a television as a display, had between 1 and 4K of RAM, and used a 16-key hexadecimal keypad for input. 
* The interpreter took up only 512 bytes of memory, and programs, which were entered into the computer in hexadecimal, were even smaller.
*
* In the early 1990s, the Chip-8 language was revived by a man named Andreas Gustafsson. He created a Chip-8 interpreter for the HP48 graphing calculator, 
* called Chip-48. The HP48 was lacking a way to easily make fast games at the time, and Chip-8 was the answer. Chip-48 later begat Super Chip-48, 
* a modification of Chip-48 which allowed higher resolution graphics, as well as other graphical enhancements.
*
* Chip-48 inspired a whole new crop of Chip-8 interpreters for various platforms, including MS-DOS, Windows 3.1, Amiga, HP48, MSX, Adam, and ColecoVision.
*
* Technical Reference(used for this emulator): http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
*
* More information: https://en.wikipedia.org/wiki/CHIP-8
* 
* @author  James Kozlowski
* @version April 2, 2017
* rust rewrite
*/

use{ 
    std::time,
};

pub struct Chip8CPU
{
    //The currently running opcode
    opcode: u16,

    /*
     Memory Map:
     +---------------+= 0xFFF (4095) End of Chip-8 RAM
     |               |
     |               |
     | 0x200 to 0xFFF|
     |     Chip-8    |
     | Program / Data|
     |     Space     |
     |               |
     |               |
     +- - - - - - - -+= 0x600 (1536) Start of ETI 660 Chip-8 programs
     |               |
     |               |
     +---------------+= 0x200 (512) Start of most Chip-8 programs
     | 0x000 to 0x1FF|
     | Reserved for  |
     |  interpreter  |
     +---------------+= 0x000 (0) Start of Chip-8 RAM
    */
    memory: [u8;4096],

    //16 general purpose 8-bit registers.
    //The VF register should not be used by any program, as it is used as a flag by some instructions.
    v: [u8;16],

    //Super Chip 8 8-bit, user-flag registers
    r: [u8;8],

    //This register is generally used to store memory addresses.
    i: u16,

    //points the the location in memory of the next Opcode
    pc: u16,

    /*
     The display memmory
     *********************
     *(0,0)        (63,0)*
     *                   *
     *(0,31)      (63,31)*
     *********************
    */
    video_memory: [u8;128 * 64],

    //Super Chip-8 extended graphics enabled
    extended_graphics_mode: bool,

    //The delay timer is active whenever the delay timer register (DT) is non-zero. 
    //This timer does nothing more than subtract 1 from the value of DT at a rate of 60Hz. When DT reaches 0, it deactivates.
    delay_timer: u8,

    //The sound timer is active whenever the sound timer register (ST) is non-zero. This timer also decrements at a rate of 60Hz, 
    //however, as long as ST's value is greater than zero, the Chip-8 buzzer will sound. When ST reaches zero, the sound timer deactivates.
    sound_timer: u8,

    //LIFO Call Stack
    stack: [u16;16],

    //Call Stack pointer
    sp: u8,

    /*
     The computers which originally used the Chip-8 Language had a 16-key hexadecimal keypad with the following layout:
     1 2 3 4
     4 5 6 D
     7 8 9 E
     A 0 B F
    */
    key: [u8;16],

    //Set to true when the screen needs to be redrawn
    //Should be set to false once the screen has been redrawn
    refresh_screen: bool,

    //Set to true if a beep needs to be played
    play_beep: bool,

    //used for chip-8 timing, please do not touch
    last_tick: i32,

    //used for chip-8 timing, please do not touch
    last_tick2: i32,
}

/**
* Returns time millicount used for get milliSpan
*
* @return millicount.
*/
fn Chip8getMilliCount() -> u32
{
    struct timeb tb;
    ftime(&tb);
    int nCount = tb.millitm + (tb.time & 0xfffff) * 1000;
    return nCount;
}

/**
* Returns time span from nTimeStart
*
* @param nTimeStart Time to measure span from
* @return time span.
*/
fn Chip8getMilliSpan(unsigned long nTimeStart) -> u32
{
    int nSpan = Chip8getMilliCount() - nTimeStart;
    if(nSpan < 0)
        nSpan += 0x100000 * 1000;
    return nSpan;
}

/**********************************************************************************************
 * This is the Chip 8 font set. Each number or character is 4 pixels wide and 5 pixel high.
 **********************************************************************************************/
const FONTSET: [u8;240] =
[ 
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80,  // F

    //Super Chip-8 Font
    0xF0, 0xF0, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0xF0, 0xF0, //0
    0x20, 0x20, 0x60, 0x60, 0x20, 0x20, 0x20, 0x20, 0x70, 0x70, //1
    0xF0, 0xF0, 0x10, 0x10, 0xF0, 0xF0, 0x80, 0x80, 0xF0, 0xF0, //2
    0xF0, 0xF0, 0x10, 0x10, 0xF0, 0xF0, 0x10, 0x10, 0xF0, 0xF0, //3
    0x90, 0x90, 0x90, 0x90, 0xF0, 0xF0, 0x10, 0x10, 0x10, 0x10, //4
    0xF0, 0xF0, 0x80, 0x80, 0xF0, 0xF0, 0x10, 0x10, 0xF0, 0xF0, //5
    0xF0, 0xF0, 0x80, 0x80, 0xF0, 0xF0, 0x90, 0x90, 0xF0, 0xF0, //6
    0xF0, 0xF0, 0x10, 0x10, 0x20, 0x20, 0x40, 0x40, 0x40, 0x40, //7
    0xF0, 0xF0, 0x90, 0x90, 0xF0, 0xF0, 0x90, 0x90, 0xF0, 0xF0, //8
    0xF0, 0xF0, 0x90, 0x90, 0xF0, 0xF0, 0x10, 0x10, 0xF0, 0xF0, //9
    0xF0, 0xF0, 0x90, 0x90, 0xF0, 0xF0, 0x90, 0x90, 0x90, 0x90, //A
    0xE0, 0xE0, 0x90, 0x90, 0xE0, 0xE0, 0x90, 0x90, 0xE0, 0xE0, //B
    0xF0, 0xF0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xF0, 0xF0, //C
    0xE0, 0xE0, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0xE0, 0xE0, //D
    0xF0, 0xF0, 0x80, 0x80, 0xF0, 0xF0, 0x80, 0x80, 0xF0, 0xF0, //E
    0xF0, 0xF0, 0x80, 0x80, 0xF0, 0xF0, 0x80, 0x80, 0x80, 0x80  //F
];

/**********************************************************************************************
 * CHIP-8 has 35 opcodes, which are all two bytes long and stored big-endian. 
 * The opcodes are listed below, in hexadecimal and with the following symbols:
 *
 * OPCODE     DISC (Instructions marked with (*) are new in SUPER-CHIP.)
 * --------------------------------------------------------------------------------------------
 * 00CN*    Scroll display N lines down
 * 0NNN        RCA 1802 program at address NNN. Not necessary for most ROMs.
 * 00E0     Clears the screen.
 * 00EE     Returns from a subroutine.
 * 00FB*    Scroll display 4 pixels right
 * 00FC*    Scroll display 4 pixels left
 * 00FD*    Exit CHIP interpreter
 * 00FE*    Disable extended screen mode
 * 00FF*    Enable extended screen mode for full-screen graphics
 * 1NNN     Jumps to address NNN.
 * 2NNN     Calls subroutine at NNN.
 * 3XNN     Skips the next instruction if VX equals NN. 
 * 4XNN     Skips the next instruction if VX doesn't equal NN. 
 * 5XY0     Skips the next instruction if VX equals VY. 
 * 6XNN     Sets VX to NN.
 * 7XNN     Adds NN to VX.
 * 8XY0     Sets VX to the value of VY.
 * 8XY1     Sets VX to VX or VY. (Bitwise OR operation) VF is reset to 0.
 * 8XY2     Sets VX to VX and VY. (Bitwise AND operation) VF is reset to 0.
 * 8XY3     Sets VX to VX xor VY. VF is reset to 0.
 * 8XY4     Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
 * 8XY5     VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
 * 8XY6     Shifts VX right by one. VF is set to the value of the least significant bit of VX before the shift.
 * 8XY7     Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
 * 8XYE     Shifts VX left by one. VF is set to the value of the most significant bit of VX before the shift.
 * 9XY0     Skips the next instruction if VX doesn't equal VY. 
 * ANNN     Sets I to the address NNN.
 * BNNN     Jumps to the address NNN plus V0.
 * CXNN     Sets VX to the result of a bitwise and operation on a random number and NN.
 * DXYN*    Show N-byte sprite from M(I) at coords (VX,VY), VF :=
 *          collision. If N=0 and extended mode, show 16x16 sprite.
 * EX9E     Skips the next instruction if the key stored in VX is pressed. 
 * EXA1     Skips the next instruction if the key stored in VX isn't pressed.
 * FX07     Sets VX to the value of the delay timer.
 * FX0A     A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event)
 * FX15     Sets the delay timer to VX.
 * FX18     Sets the sound timer to VX.
 * FX1E     Adds VX to I.[3]
 * FX29     sets I to the location of the sprite for the character in VX. Characters 0-F  are represented by a 4x5 font.
 * FX30*    Point I to 10-byte font sprite for digit VX (0..9)
 * FX33     Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I.
 * FX55     Stores V0 to VX (including VX) in memory starting at address I.[4]
 * FX65     Fills V0 to VX (including VX) with values from memory starting at address I.
 * FX75*    Store V0..VX in RPL user flags (X <= 7)
 * FX85*    Read V0..VX from RPL user flags (X <= 7) 
 **********************************************************************************************/

//Array of function pointers to the OpCodes
const OPCODE_TABLE: [fn(&mut Chip8CPU);16] = 
[
    Chip8OpCode00XX,
    Chip8OpCode1NNN,
    Chip8OpCode2NNN,
    Chip8OpCode3XNN,
    Chip8OpCode4XNN,
    Chip8OpCode5XY0,
    Chip8OpCode6XN0,
    Chip8OpCode7XNN, 
    Chip8ARITHMETIC,
    Chip8OpCode9XY0, 
    Chip8OpCodeANNN, 
    Chip8OpCodeBNNN, 
    Chip8OpCodeCXKK, 
    Chip8OpCodeDXYN, 
    Chip8OpCodeEXXX, 
    Chip8OpCodeFXXX,        //hard to split this one up in a meaningfull way
];

//Array of Function pointers to the 8???? OpCodes
const ARITHMETIC_TABLE: [fn(&mut Chip8CPU);16] = 
[
    Chip8OpCode8XY0,
    Chip8OpCode8XY1,
    Chip8OpCode8XY2,
    Chip8OpCode8XY3,
    Chip8OpCode8XY4,
    Chip8OpCode8XY5,
    Chip8OpCode8XY6,
    Chip8OpCode8XY7,
    Chip8CPUNULL,
    Chip8CPUNULL,
    Chip8CPUNULL,
    Chip8CPUNULL,
    Chip8CPUNULL,
    Chip8CPUNULL,
    Chip8OpCode8XYE,
    Chip8CPUNULL,
];
    

/**
* Resets the Chip8CPU to power on defaults
* Loads the default font set into memory
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
fn Chip8Reset() -> Chip8CPU
{
    let mut c8 = Chip8CPU{
        pc: 0x200,                  // Program counter starts at 0x200
        sp:  0,
        i:  0,
        delay_timer: 0,
        sound_timer: 0,
        refresh_screen: false,
        play_beep: false,
        last_tick: 0,
        lastTick2: 0,
        
        v:[0; 16],
        r:[0; 8],
        key [0; 16],
        video_memory: [0; 8192],
        memory: [0;4096],
        stack: [0;16],

        refreshScreen: false,
    }
    for i in 0..240{
        c8.memory[i] = FONTSET[i];
    }
    c8
}

/**
* Loads the Chip8 ROM into memory starting at 0x200
*
* @param Chip8 Address of the Chip8CPU object
* @param filename filename to load
* @return false if the file could not be loaded.
*/
fn Chip8LoadRom(Chip8: &mut Chip8CPU,filename: String) -> bool
{
    FILE *file;
    file = fopen(filename,"rb");
    
    if (!file)
        return false;
    
    let mut b: u8;
    let mut i: i32 = 0x200;

    while (!feof(file))
    {
        fread(&b, sizeof(char), 1, file);
        //printf("%i", b);
        Chip8.memory[i] = b;
        i+=1;
    }
    
    fclose(file);

    return true;
}

/**
* Saves the emulator state to a file
*
* @param Chip8 Address of the Chip8CPU object
* @param filename filename to save state to
* @return false if the file could not be saved.
*/
bool Chip8SaveState(Chip8CPU *Chip8, char *filename)
{
    FILE *file;
    file = fopen(filename,"wb");
    
    if (!file)
        return false;
    
    fwrite(Chip8, sizeof(Chip8CPU), 1, file);
    
    fclose(file);

    return true;
}

/**
* Loads the emulator state to a file
*
* @param Chip8 Address of the Chip8CPU object
* @param filename filename to Load state from
* @return false if the file could not be loaded.
*/
bool Chip8LoadState(Chip8CPU *Chip8, char *filename)
{
    FILE *file;
    file = fopen(filename,"rb");
    
    if (!file)
        return false;
    
       fread(Chip8, sizeof(Chip8CPU), 1, file);
    
    fclose(file);

    return true;
}

/**
* Fetches and runs a opcode
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
fn Chip8EmulateCycle(Chip8: &mut Chip8CPU)
{
    if (Chip8getMilliSpan(c8.last_tick) < 1){
        return;
    }
    c8.last_tick = Chip8getMilliCount();

    c8.opcode = Chip8->memory[Chip8->pc++] << 8 | Chip8->memory[Chip8->pc++];
    
    //printf("opcode: %04X\n", Chip8->opcode );
    
    (*Chip8OpcodeTable[(Chip8->opcode&0xF000)>>12])(Chip8);

    if (Chip8getMilliSpan(Chip8->lastTick2) > 6)
    {
        Chip8->lastTick2 = Chip8getMilliCount();
        if(Chip8->delayTimer > 0)
            --Chip8->delayTimer;
 
        if(Chip8->soundTimer > 0)
        {
            if(Chip8->soundTimer == 1)
                Chip8->playBeep = true;
            --Chip8->soundTimer;
        }
    }
}

/*************************************************************************************************
 * opcodes
*************************************************************************************************/

/**
* If this OPCODE is called then something went wrong
* or the opcode was not finished
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
fn Chip8CPUNULL(Chip8: &mut Chip8CPU)
{
    println!("bad opcode: {:#06x} at: {:#06x}", c8.opcode, c8.pc );
}


/**
* 0000 opcodes
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
fn Chip8OpCode00XX(c8: &mut Chip8CPU)
{
    //odd one out
    if c8.opcode & 0x00F0 == 0x00C0{
        Chip8OpCode00CN(&Chip8);
    }
    else
    {
        match c8.opcode & 0x00FF
        {
            0x00E0 => Chip8OpCode00E0(&Chip8),
            0x00EE => Chip8OpCode00EE(&Chip8),
            0x00FB => Chip8OpCode00FB(&Chip8),
            0x00FC => Chip8OpCode00FC(&Chip8),
            0x00FD => Chip8OpCode00FD(&Chip8),
            0x00FE => Chip8OpCode00FE(&Chip8),
            0x00FF => Chip8OpCode00FF(&Chip8),
            _ => Chip8CPUNULL(&Chip8),
        }
    }
}

/**
* 00E0 - Clear the display.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
fn Chip8OpCode00E0(c8:&mut Chip8CPU)
{
    
    c8.video_memory = [0;8192];
    //Chip8->refreshScreen = true;
}

/**
* Scroll display N lines down
* Moves the memory pointer
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode00CN(Chip8CPU *Chip8)
{
    
    int screenx = 64;
    int screeny = 32;

    if (Chip8->extendedGraphicsMode == true)
    {
        screenx = 128;
        screeny = 64;
    }

    int n = (Chip8->opcode & 0x000F);

    for (int i = screeny; i >= n; --i)
    {
        int start = i *screeny;
        for (auto j = start; j < (start + screenx); ++j)
        {
            Chip8->videoMemory[j] = Chip8->videoMemory[j - (screenx * n)];
        }
    }

    memset(Chip8->videoMemory, 0, (n) * screenx);
    //Chip8->refreshScreen = true;
}

/**
* 00EE - Return from a subroutine. 
* The interpreter sets the program counter to the address at the top of the stack,
* then subtracts 1 from the stack pointer.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
fn Chip8OpCode00EE(c8: &mut Chip8CPU)
{
    c8.sp-=1;
    c8.pc = c8.stack[c8.sp];
}

/**
* Scroll display 4 pixels right (Super Chip-8)
* 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode00FB(Chip8CPU *Chip8)
{

    int screenx = 64;
    int screeny = 32;

    if (Chip8->extendedGraphicsMode == true)
    {
        screenx = 128;
        screeny = 64;
    }

    for (int i = 0; i < screeny; ++i)

    {
        int start = i * screenx;
        for (auto j = start + (screenx - 1); j >= (start + 4); --j)
        {
            Chip8->videoMemory[j] = Chip8->videoMemory[j - 4];
        }

        memset(Chip8->videoMemory + start, 0, 4);
    }
    Chip8->refreshScreen = true;
}

/**
* Scroll display 4 pixels left (Super Chip-8)
* 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode00FC(Chip8CPU *Chip8)
{

    int screenx = 64;
    int screeny = 32;

    if (Chip8->extendedGraphicsMode == true)
    {
        screenx = 128;
        screeny = 64;
    }

    for (auto i = 0; i < screeny; ++i)
    {
        auto start = i * screenx;
        for (auto j = start; j < start + (screenx - 4); ++j)
        {
            Chip8->videoMemory[j] = Chip8->videoMemory[j + 4];
        }

        memset(Chip8->videoMemory + start + (screenx - 5), 0, 4);
    }
    Chip8->refreshScreen = true;
}

/**
* Exit CHIP interpreter (Super Chip-8)
* 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode00FD(Chip8CPU *Chip8)
{
    //dont really do anything hre, reset i guess
    Chip8Reset(Chip8);
}

/**
* Disable extended screen mode (Super Chip-8)
* 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode00FE(Chip8CPU *Chip8)
{
    Chip8->extendedGraphicsMode = false;
}

/**
* Enable extended screen mode for full-screen graphics (Super Chip-8)
* 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
fn Chip8OpCode00FF(c8:&mut Chip8CPU)
{
    c8.extended_graphics_mode = true;
}

/**
* Jump to location nnn.
* The interpreter sets the program counter to nnn.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode1NNN(Chip8CPU *Chip8)
{
    Chip8->pc = Chip8->opcode & 0x0FFF;
}

/**
* Call subroutine at nnn.
* The interpreter increments the stack pointer, 
* then puts the current PC on the top of the stack. 
* The PC is then set to nnn.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode2NNN(Chip8CPU *Chip8)
{
    Chip8->stack[Chip8->sp++] = Chip8->pc;
    Chip8->pc = Chip8->opcode & 0x0FFF;
}

/**
* Skip next instruction if Vx = kk.
* The interpreter compares register Vx to kk, 
* and if they are equal, increments the program counter by 2.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode3XNN(Chip8CPU *Chip8)
{
    if (Chip8->V[(Chip8->opcode & 0x0F00) >> 8] == (Chip8->opcode & 0x00FF) )
        Chip8->pc += 2;
}

/**
* Skip next instruction if Vx != kk.
* The interpreter compares register Vx to kk, 
* and if they are not equal, increments the program counter by 2.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode4XNN(Chip8CPU *Chip8)
{
    if (Chip8->V[(Chip8->opcode & 0x0F00) >> 8] != (Chip8->opcode & 0x00FF) )
        Chip8->pc += 2;
}

/**
* Skip next instruction if Vx != Vy.
* The interpreter compares register Vx to Vy, 
* and if they are equal, increments the program counter by 2.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode5XY0(Chip8CPU *Chip8)
{
    if (Chip8->V[(Chip8->opcode & 0x0F00) >> 8] == Chip8->V[(Chip8->opcode & 0x00F0) >> 4] )  
        Chip8->pc += 2;
}

/**
* Set Vx = kk.
* The interpreter puts the value kk into register Vx.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode6XN0(Chip8CPU *Chip8)
{
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] = (Chip8->opcode & 0x00FF);
}

/**
* Set Vx = Vx + kk.
* Adds the value kk to the value of register Vx, 
* then stores the result in Vx. 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode7XNN(Chip8CPU *Chip8)
{
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] += (Chip8->opcode & 0x00FF);
}

/**
* Calls the 8??? op codes
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8ARITHMETIC(Chip8CPU *Chip8)
{
    Chip8ArithmeticOpcodeTable[(Chip8->opcode&0x000F)](Chip8);
}

/**
* Set Vx = Vy.
* Stores the value of register Vy in register Vx. 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode8XY0(Chip8CPU *Chip8)
{
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] = Chip8->V[(Chip8->opcode & 0x00F0) >> 4]; 
}

/**
* Set Vx = Vx OR Vy.
* Performs a bitwise OR on the values of Vx and Vy, 
* then stores the result in Vx. 
* A bitwise OR compares the corrseponding bits from two values, 
* and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0. 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode8XY1(Chip8CPU *Chip8)
{
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] = Chip8->V[(Chip8->opcode & 0x0F00) >> 8] | Chip8->V[(Chip8->opcode & 0x00F0) >> 4];  
}

/**
* Set Vx = Vx AND Vy.
* Performs a bitwise AND on the values of Vx and Vy, 
* then stores the result in Vx.
* A bitwise AND compares the corrseponding bits from two values,
* and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0. 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode8XY2(Chip8CPU *Chip8)
{
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] = Chip8->V[(Chip8->opcode & 0x0F00) >> 8] & Chip8->V[(Chip8->opcode & 0x00F0) >> 4];
}

/**
* Set Vx = Vx XOR Vy.
* Performs a bitwise exclusive OR on the values of Vx and Vy, 
* then stores the result in Vx. 
* An exclusive OR compares the corrseponding bits from two values, 
* and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0. 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode8XY3(Chip8CPU *Chip8)
{
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] = Chip8->V[(Chip8->opcode & 0x0F00) >> 8] ^ Chip8->V[(Chip8->opcode & 0x00F0) >> 4];
}

/**
* Set Vx = Vx + Vy, set VF = carry.
* The values of Vx and Vy are added together. 
* If the result is greater than 8 bits (i.e., > 255,) VF is set to 1,
* otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode8XY4(Chip8CPU *Chip8)
{
    if (Chip8->V[(Chip8->opcode & 0x00F0) >> 4] > (0xFF - Chip8->V[(Chip8->opcode & 0x0F00) >> 8]))
        Chip8->V[0xF] = 1; //carry
    else
        Chip8->V[0xF] = 0;
    
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] += Chip8->V[(Chip8->opcode & 0x00F0) >> 4];
}

/**
* Set Vx = Vx - Vy, set VF = NOT borrow.
* If Vx > Vy, then VF is set to 1, otherwise 0. 
* Then Vy is subtracted from Vx, and the results stored in Vx.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode8XY5(Chip8CPU *Chip8)
{
    if (Chip8->V[(Chip8->opcode & 0x00F0) >> 4] > Chip8->V[(Chip8->opcode & 0x0F00) >> 8]) 
        Chip8->V[0xF] = 0; // there is a borrow
    else 
        Chip8->V[0xF] = 1;                    
    
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] -= Chip8->V[(Chip8->opcode & 0x00F0) >> 4];
}

/**
* Set Vx = Vx SHR 1.
* If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. 
* Then Vx is divided by 2.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode8XY6(Chip8CPU *Chip8)
{
    Chip8->V[0xF] = (Chip8->V[(Chip8->opcode & 0x0F00) >> 8]) & 0x1;
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] >>= 1;
}

/**
* Set Vx = Vy - Vx, set VF = NOT borrow.
* If Vy > Vx, then VF is set to 1, otherwise 0. 
* Then Vx is subtracted from Vy, and the results stored in Vx.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode8XY7(Chip8CPU *Chip8)
{
    if (Chip8->V[(Chip8->opcode & 0x0F00) >> 8] > Chip8->V[(Chip8->opcode & 0x00F0) >> 4])    // VY-VX
        Chip8->V[0xF] = 0; // there is a borrow
    else
        Chip8->V[0xF] = 1;
    
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] = Chip8->V[(Chip8->opcode & 0x00F0) >> 4] - Chip8->V[(Chip8->opcode & 0x0F00) >> 8];                                    
}

/**
* Set Vx = Vx SHL 1.
* If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. 
* Then Vx is multiplied by 2.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode8XYE(Chip8CPU *Chip8)
{
    Chip8->V[0xF] = Chip8->V[(Chip8->opcode & 0x0F00) >> 8] >> 7;
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] <<= 1;
}

/**
* Skip next instruction if Vx != Vy.
* The values of Vx and Vy are compared, 
* and if they are not equal, the program counter is increased by 2.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCode9XY0(Chip8CPU *Chip8)
{
    if (Chip8->V[(Chip8->opcode & 0x0F00) >> 8] != Chip8->V[(Chip8->opcode & 0x00F0) >> 4])
        Chip8->pc += 2;
}

/**
* Set I = nnn.
* The value of register I is set to nnn.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeANNN(Chip8CPU *Chip8)
{
    Chip8->I = Chip8->opcode & 0x0FFF;
}

/**
* Jump to location nnn + V0.
* The program counter is set to nnn plus the value of V0.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeBNNN(Chip8CPU *Chip8)
{
    Chip8->pc = (Chip8->opcode & 0x0FFF) + Chip8->V[0];
}

/**
* Set Vx = random byte AND kk.
* The interpreter generates a random number from 0 to 255, 
* which is then ANDed with the value kk. 
* The results are stored in Vx. 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeCXKK(Chip8CPU *Chip8)
{
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] = (rand() % 0xFF) & (Chip8->opcode & 0x00FF);
}

/**
* Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
* The interpreter reads n bytes from memory, starting at the address stored in I. 
* These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). 
* Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, 
* otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, 
* it wraps around to the opposite side of the screen.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeDXYN(Chip8CPU *Chip8)
{
    unsigned short x = Chip8->V[(Chip8->opcode & 0x0F00) >> 8];
    unsigned short y = Chip8->V[(Chip8->opcode & 0x00F0) >> 4];
    unsigned short height = Chip8->opcode & 0x000F;
    unsigned short pixel;
    Chip8->V[0xF] = 0;

    int screenx = 64;
    int screeny = 32;

    if (Chip8->extendedGraphicsMode == true)
    {
        screenx = 128;
        screeny = 64;
    }

    //super Chip-8
    if (height == 0)
    {
        height = 32;
        for (int yline = 0; yline < height; yline += 2)
        {
            //pixel = Chip8->memory[Chip8->I + yline];
            pixel = (Chip8->memory[Chip8->I + yline] << 8) | Chip8->memory[Chip8->I + yline + 1];
            for (int xline = 0; xline < 16; xline++)
            {
                if ((pixel & (0x8000 >> xline)) != 0)
                {
                    if (Chip8->videoMemory[(x + xline + ((y + (yline / 2)) * screenx))] == 1)
                    {
                        Chip8->V[0xF] = 1;                                    
                    }
                    Chip8->videoMemory[(x + xline + ((y + (yline / 2)) * screenx))] ^= 1;
                }
            }
        }
    }
    //Chip-8
    else
    {
        for (int yline = 0; yline < height; yline++)
        {
            pixel = Chip8->memory[Chip8->I + yline];
            for (int xline = 0; xline < 8; xline++)
            {
                if ((pixel & (0x80 >> xline)) != 0)
                {
                    if (Chip8->videoMemory[(x + xline + ((y + yline) * screenx))] == 1)
                    {
                        Chip8->V[0xF] = 1;                                    
                    }
                    Chip8->videoMemory[x + xline + ((y + yline) * screenx)] ^= 1;
                }
            }
        }
    }
    Chip8->refreshScreen = true;
}

/**
* Skip next instruction if key with the value of Vx is pressed.
* Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, 
* PC is increased by 2.
* OR
* Skip next instruction if key with the value of Vx is not pressed.
* Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, 
* PC is increased by 2.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeEXXX(Chip8CPU *Chip8)
{
    switch(Chip8->opcode & 0x00FF)
    {
        case 0x009E: // EX9E     Skips the next instruction if the key stored in VX is pressed.
            if(Chip8->key[Chip8->V[(Chip8->opcode & 0x0F00) >> 8]] != 0)
                Chip8->pc += 2;
            break;
        case 0x0A1: // EXA1     Skips the next instruction if the key stored in VX isn't pressed.
            if(Chip8->key[Chip8->V[(Chip8->opcode & 0x0F00) >> 8]] == 0)
                Chip8->pc += 2;
            break;
        default:
            Chip8CPUNULL(Chip8);     
    }    
}

/**
* A Switch to call the FXXX OpCodes
* I dont really see a better way to seperate them out.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFXXX(Chip8CPU *Chip8)
{
    switch(Chip8->opcode & 0x00FF)
    {
        case 0x0007: 
            Chip8OpCodeFX07(Chip8);
            break;
        
        case 0x000A: 
            Chip8OpCodeFX0A(Chip8);
            break;
        
        case 0x0015: 
            Chip8OpCodeFX15(Chip8);
            break;
        
        case 0x0018: 
            Chip8OpCodeFX18(Chip8);
            break;
        
        case 0x001E: 
            Chip8OpCodeFX1E(Chip8);
            break;
        
        case 0x0029: 
            Chip8OpCodeFX29(Chip8);
            break;
        
        case 0x0030:
            Chip8OpCodeFX30(Chip8);
            break;

        case 0x0033: 
            Chip8OpCodeFX33(Chip8);
            break;
            
        case 0x0055: 
            Chip8OpCodeFX55(Chip8);
            break;
        
        case 0x0065: 
            Chip8OpCodeFX65(Chip8);
            break;
        
        case 0x0075: 
            Chip8OpCodeFX75(Chip8);
            break;

        case 0x0085: 
            Chip8OpCodeFX85(Chip8);
            break;
        
        default:
            Chip8CPUNULL(Chip8);
    }    
}

/**
* Set Vx = delay timer value.
* The value of DT is placed into Vx.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX07(Chip8CPU *Chip8)
{
    Chip8->V[(Chip8->opcode & 0x0F00) >> 8] = Chip8->delayTimer;
}

/**
* Wait for a key press, store the value of the key in Vx.
* All execution stops until a key is pressed, then the value of that key is stored in Vx.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX0A(Chip8CPU *Chip8)
{
    bool keyPress = false;

    for(int i = 0; i < 16; ++i)
    {
        if(Chip8->key[i] != 0)
        {
            Chip8->V[(Chip8->opcode & 0x0F00) >> 8] = i;
            keyPress = true;
        }
    }
    // If we didn't received a keypress, skip this cycle and try again.
    if(!keyPress)                        
        Chip8->pc -= 2;
}

/**
* Set delay timer = Vx.
* DT is set equal to the value of Vx.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX15(Chip8CPU *Chip8)
{
    Chip8->delayTimer = Chip8->V[(Chip8->opcode & 0x0F00) >> 8];
}

/**
* Set sound timer = Vx.
* ST is set equal to the value of Vx.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX18(Chip8CPU *Chip8)
{
    Chip8->soundTimer = Chip8->V[(Chip8->opcode & 0x0F00) >> 8];
}

/**
* Set I = I + Vx.
* The values of I and Vx are added, and the results are stored in I.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX1E(Chip8CPU *Chip8)
{
    if (Chip8->I + Chip8->V[(Chip8->opcode & 0x0F00) >> 8] > 0xFFF)    // VF is set to 1 when range overflow (I+VX>0xFFF), and 0 when there isn't.
        Chip8->V[0xF] = 1;
    else
        Chip8->V[0xF] = 0;

    Chip8->I += Chip8->V[(Chip8->opcode & 0x0F00) >> 8];
}

/**
* Set I = location of sprite for digit Vx.
* The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX29(Chip8CPU *Chip8)
{
    Chip8->I = Chip8->V[(Chip8->opcode & 0x0F00) >> 8] * 0x5;
}

/**
* Set I = location of 10 bit sprite for digit Vx. (Super Chip-8)
* The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. 
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX30(Chip8CPU *Chip8)
{
    Chip8->I = Chip8->V[(Chip8->opcode & 0x0F00) >> 8] * 0x5;
}

/**
* Store BCD representation of Vx in memory locations I, I+1, and I+2.
* The interpreter takes the decimal value of Vx, 
* and places the hundreds digit in memory at location in I, 
* the tens digit at location I+1, and the ones digit at location I+2.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX33(Chip8CPU *Chip8)
{
    Chip8->memory[Chip8->I]     = (Chip8->V[(Chip8->opcode & 0x0F00) >> 8] / 100);
    Chip8->memory[Chip8->I + 1] = (Chip8->V[(Chip8->opcode & 0x0F00) >> 8] / 10) % 10;
    Chip8->memory[Chip8->I + 2] = (Chip8->V[(Chip8->opcode & 0x0F00) >> 8] % 100) % 10;
}

/**
* Store registers V0 through Vx in memory starting at location I.
* The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX55(Chip8CPU *Chip8)
{
    for (int i = 0; i <= (Chip8->opcode & 0x0F00) >> 8; ++i)
        Chip8->memory[Chip8->I + i] = Chip8->V[i];
    
    Chip8->I += ((Chip8->opcode & 0x0F00) >> 8) + 1;
}

/**
* Read registers V0 through Vx from memory starting at location I.
* The interpreter reads values from memory starting at location I into registers V0 through Vx.
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
fn Chip8OpCodeFX65(c8:&mut Chip8CPU)
{
    for i in 0..(c8.opcode & 0x0F00) >> 8)+1{
        c8.v[i] = c8.memory[c8.i + i];
    }
    c8.i += ((c8.opcode & 0x0F00) >> 8) + 1;
}

/**
* Store V0..VX in R user flags (X <= 7) (Super Chip-8)
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX75(Chip8CPU *Chip8)
{
    for (int i = 0; i <= ((Chip8->opcode & 0x0F00) >> 8); i++ )
    {
        Chip8->R[i] = Chip8->V[i];
    }
}

/**
* Read V0..VX in R user flags (X <= 7) (Super Chip-8)
*
* @param Chip8 Address of the Chip8CPU object
* @return Nothing.
*/
void Chip8OpCodeFX85(Chip8CPU *Chip8)
{
    for (int i = 0; i <= ((Chip8->opcode & 0x0F00) >> 8); i++ )
    {
        Chip8->V[i] = Chip8->R[i];
    }
}


