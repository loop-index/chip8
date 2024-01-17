# A Chip8 Emulator in Rust
I wrote this little emulator because I'm still riding the high from last semester's CompArch class, and also because I wanted to try out Rust.

<img src="./demo.gif" alt="Demo of the emulator">

## Usage
There are three programs in this package, the emulator and an assembler/disassembler for the instruction set.

### The Emulator
You can run the main emulator with cargo:

<code> cargo run --bin chip8 -- --rom <PATH/TO/ROM></code>

Additional arguments:
<code>-f, --frames <FRAMERATE>       Number of frames to render per second [default: 100]
-r, --rom <ROM>                The ROM to load
-c, --cycles <TICK_PER_FRAME>  Number of instructions to execute per frame [default: 8]
    --no-keypad                Disable keypad rendering
    --smpte                    Enable SMPTE color mode
-h, --help                     Print help</code>

### The Assembler
The program takes a text file and outputs a hex file. Usage:
<code> cargo run --bin asm <PATH/TO/SOURCE> <PATH/TO/OUTPUT> </code>

### The Disassembler
Similarly, the program takes a hex file and outputs a text file. Usage:
<code> cargo run --bin deasm <PATH/TO/SOURCE> <PATH/TO/OUTPUT> </code>

## Acknowledgements
The included ROM folder is taken from https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html

Some code is inspired by https://github.com/cmleon51/cli-chip8-emulator.rs