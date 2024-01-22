/*
* Rust Chip8 Emulator
* Author: Duong Nguyen
* Credits:
*   - https://github.com/cmleon51/cli-chip8-emulator.rs for the keyboard polling fix
*/

use crate::chip8::*;
pub mod chip8;

use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{event, terminal};
use std::time::Duration;
use std::thread;
use clap::Parser;

// Here I use the Braille character set to represent pixels.
// A Braille character can be mapped to binary, with the bottom right dot being the least significant bit. In this way, I can place each character at the index that it represents, which can easily be indexed into based on the screen data.
const BRAILLE_MAP: [char; 256] = [
    '⠀', '⢀', '⠠', '⢠', '⠐', '⢐', '⠰', '⢰', 
    '⠈', '⢈', '⠨', '⢨', '⠘', '⢘', '⠸', '⢸', 
    '⡀', '⣀', '⡠', '⣠', '⡐', '⣐', '⡰', '⣰', 
    '⡈', '⣈', '⡨', '⣨', '⡘', '⣘', '⡸', '⣸', 
    '⠄', '⢄', '⠤', '⢤', '⠔', '⢔', '⠴', '⢴', 
    '⠌', '⢌', '⠬', '⢬', '⠜', '⢜', '⠼', '⢼', 
    '⡄', '⣄', '⡤', '⣤', '⡔', '⣔', '⡴', '⣴', 
    '⡌', '⣌', '⡬', '⣬', '⡜', '⣜', '⡼', '⣼', 
    '⠂', '⢂', '⠢', '⢢', '⠒', '⢒', '⠲', '⢲', 
    '⠊', '⢊', '⠪', '⢪', '⠚', '⢚', '⠺', '⢺', 
    '⡂', '⣂', '⡢', '⣢', '⡒', '⣒', '⡲', '⣲', 
    '⡊', '⣊', '⡪', '⣪', '⡚', '⣚', '⡺', '⣺', 
    '⠆', '⢆', '⠦', '⢦', '⠖', '⢖', '⠶', '⢶', 
    '⠎', '⢎', '⠮', '⢮', '⠞', '⢞', '⠾', '⢾', 
    '⡆', '⣆', '⡦', '⣦', '⡖', '⣖', '⡶', '⣶', 
    '⡎', '⣎', '⡮', '⣮', '⡞', '⣞', '⡾', '⣾', 
    '⠁', '⢁', '⠡', '⢡', '⠑', '⢑', '⠱', '⢱', 
    '⠉', '⢉', '⠩', '⢩', '⠙', '⢙', '⠹', '⢹', 
    '⡁', '⣁', '⡡', '⣡', '⡑', '⣑', '⡱', '⣱', 
    '⡉', '⣉', '⡩', '⣩', '⡙', '⣙', '⡹', '⣹', 
    '⠅', '⢅', '⠥', '⢥', '⠕', '⢕', '⠵', '⢵', 
    '⠍', '⢍', '⠭', '⢭', '⠝', '⢝', '⠽', '⢽', 
    '⡅', '⣅', '⡥', '⣥', '⡕', '⣕', '⡵', '⣵', 
    '⡍', '⣍', '⡭', '⣭', '⡝', '⣝', '⡽', '⣽', 
    '⠃', '⢃', '⠣', '⢣', '⠓', '⢓', '⠳', '⢳', 
    '⠋', '⢋', '⠫', '⢫', '⠛', '⢛', '⠻', '⢻', 
    '⡃', '⣃', '⡣', '⣣', '⡓', '⣓', '⡳', '⣳', 
    '⡋', '⣋', '⡫', '⣫', '⡛', '⣛', '⡻', '⣻', 
    '⠇', '⢇', '⠧', '⢧', '⠗', '⢗', '⠷', '⢷', 
    '⠏', '⢏', '⠯', '⢯', '⠟', '⢟', '⠿', '⢿', 
    '⡇', '⣇', '⡧', '⣧', '⡗', '⣗', '⡷', '⣷', 
    '⡏', '⣏', '⡯', '⣯', '⡟', '⣟', '⡿', '⣿',
];

/// A struct to clean up the terminal when the program exits/panics
struct CleanUp;

/// Implement Drop trait for CleanUp, which will be called when the struct goes out of scope
impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not disable raw mode");

        // Enable cursor
        print!("\x1b[?25h");

        if std::thread::panicking() {
            println!("Panic! at the disco");
        }
    }
}

/// Struct to hold the arguments passed to the program
#[derive(Parser, Debug)]
#[command(about = "A Chip8 emulator written in Rust")]
struct Args {
    /// Number of frames to render per second
    #[arg(short='f', long="frames", default_value="100")]
    framerate: u64,

    /// The ROM to load
    #[arg(short, long)]
    rom: String,

    /// Number of instructions to execute per frame
    #[arg(short='c', long="cycles", default_value="8")]
    tick_per_frame: u64,

    /// Disable keypad rendering
    #[arg(long="no-keypad", action)]
    no_keypad: bool,

    /// Enable SMPTE color mode
    #[arg(long="smpte", action)]
    smpte: bool,
}

fn main() {
    // Read arguments
    let args = Args::parse();

    // Check terminal size
    let (_width, height) = terminal::size().expect("Failed to get terminal size");

    // Weirdly here shifting right drops the entire value to 0, so I have to use division instead. I hope the compiler optimizes this :(
    if !args.no_keypad && height < SCREEN_HEIGHT as u16 / 4 + 14 {
        if height >= SCREEN_HEIGHT as u16 / 4 + 5 {
            println!("Terminal height is too small, which might lead to rendering issues. Please resize the terminal to have at least {} rows, or consider running with --no-keypad flag on.", SCREEN_HEIGHT / 4 + 14);
        } else {
            println!("Terminal height is too small, which might lead to rendering issues. Please resize the terminal to have at least {} rows.", SCREEN_HEIGHT / 4 + 14);
        }
        return;
    } else if args.no_keypad && height < SCREEN_HEIGHT as u16 / 4 + 5 {
        println!("Terminal height is too small, which might lead to rendering issues. Please resize the terminal to have at least {} rows.", SCREEN_HEIGHT / 4 + 5);
        return;
    }

    let frame_duration: u64 = 1000 / args.framerate;

    // Prepare the terminal
    let _clean_up = CleanUp;
    terminal::enable_raw_mode().expect("Failed to enable raw mode");

    // Disable cursor
    print!("\x1b[?25l");

    // Load the ROM
    let rom = std::fs::read(&args.rom).expect("Failed to read ROM");

    // Create the Chip8
    let mut chip8 = Chip8::new();

    // Load the ROM into memory
    chip8.load_rom(&rom);

    // Display instructions
    println!("\rRunning ROM {} ({} bytes) at {} FPS", args.rom, rom.len(), args.framerate);
    println!("\rKeybindings:");
    println!("\r\t1 2 3 4");
    println!("\r\tq w e r");
    println!("\r\ta s d f");
    println!("\r\tz x c v");
    println!("\rPress Esc to quit");
    println!("\rPress any key to start");
    event::read().expect("Failed to read line");
    print!("\x1b[2J\x1b[1;1H");

    // Main loop
    'main_loop: loop {
        // Clear keypresses
        chip8.clear_keypad();

        for _ in 0..args.tick_per_frame {
            // Poll for events
            if event::poll(Duration::from_micros(1)).expect("Error") {
                if let Event::Key(event) = event::read().expect("Failed to read line") {
                    match event {
                        KeyEvent {
                            ..
                        } => {
                            match event.code {
                                // Quit
                                KeyCode::Esc => {
                                    break 'main_loop;
                                },
                                _ => {
                                    if let Some(button) = map_key_to_button(event.code) {
                                        chip8.set_keypress(button);
                                    }
                                }
                            }
                        },
                    }
                };
            }
        
            // Tick the Chip8
            chip8.cycle();
        }

        // Update the timers
        chip8.update_timers();
        
        // Clear the screen
        print!("\x1b[2J\x1b[1;1H");

        // Draw the screen
        draw(&chip8, &args);

        // Sleep for a bit
        thread::sleep(Duration::from_millis(frame_duration));
    }
}

/// Characters to be rendered onto the keypad
const KEY_ORDER: [char; 16] = [
    '1', '↑', '3', 'C',
    '←', '5', '→', 'D',
    '7', '↓', '9', 'E',
    'A', '0', 'B', 'F',
];

/// Hexadecimal order of the keys
const KEY_ORDER_HEX: [usize; 16] = [
    0x1, 0x2, 0x3, 0xC,
    0x4, 0x5, 0x6, 0xD,
    0x7, 0x8, 0x9, 0xE,
    0xA, 0x0, 0xB, 0xF,
];

/// SMPTE color codes
const SMPTE_COLORS: [&str; 8] = [
    "\x1b[37m", "\x1b[33m", "\x1b[36m", "\x1b[32m",
    "\x1b[35m", "\x1b[31m", "\x1b[34m", "\x1b[37m",
];

/// Draw the screen using Braille characters (innovative, right?)
/// 
/// Each character represents a 2x4 block of pixels, with the bottom right dot being the least significant bit.
/// 
/// ## Arguments
/// 
/// * `chip` - The Chip8 to draw
/// * `args` - The arguments passed to the program
fn draw(chip: &Chip8, args: &Args) {
    // Draw the outside border
    print!("╭");
    print!("─CHIP-8");
    for _ in 0..((SCREEN_WIDTH / 2) - 12) {
        print!("─");
    }
    print!("BEEP─");
    if chip.get_sound_timer() > 0 {
        print!("●─");
    } else {
        print!("○─");
    }
    println!("╮\r");

    // Draw the top border
    print!("│╭");
    for _ in 0..SCREEN_WIDTH / 2 {
        print!("─");
    }
    println!("╮│\r");

    // Draw the screen in blocks of 2x4
    let buffer = chip.get_screen_buffer();
    let mut color_ptr: usize = 0;
    for y in 0..SCREEN_HEIGHT / 4 {
        // Draw the left border
        print!("││");

        // Draw the screen
        for x in 0..SCREEN_WIDTH / 2 {
            let encoding = 
                buffer[y * 4 * SCREEN_WIDTH + x * 2] << 7 |
                buffer[y * 4 * SCREEN_WIDTH + x * 2 + 1] << 3 |
                buffer[(y * 4 + 1) * SCREEN_WIDTH + x * 2] << 6 |
                buffer[(y * 4 + 1) * SCREEN_WIDTH + x * 2 + 1] << 2 |
                buffer[(y * 4 + 2) * SCREEN_WIDTH + x * 2] << 5 |
                buffer[(y * 4 + 2) * SCREEN_WIDTH + x * 2 + 1] << 1 |
                buffer[(y * 4 + 3) * SCREEN_WIDTH + x * 2] << 4 |
                buffer[(y * 4 + 3) * SCREEN_WIDTH + x * 2 + 1];

            // Set the color
            if args.smpte && x % 4 == 0 {
                print!("{}", SMPTE_COLORS[color_ptr]);
                color_ptr = (color_ptr + 1) % 8;
            }
            print!("{}", BRAILLE_MAP[encoding as usize]);
        }

        // Reset the color
        print!("\x1b[0m");

        // Draw the right border
        println!("││\r");
    }

    // Draw the bottom border
    print!("│╰");
    for _ in 0..SCREEN_WIDTH / 2 {
        print!("─");
    }
    println!("╯│\r");

    // Draw the keypad
    if !args.no_keypad {
        let keypad = chip.get_keypad();
        // Draw the top border
        print!("│");
        for _ in 0..((SCREEN_WIDTH / 4) - 9) {
            print!(" ");
        }
        print!("╭───╮╭───╮╭───╮╭───╮");
        for _ in 0..((SCREEN_WIDTH / 4) - 9) {
            print!(" ");
        }
        println!("│\r");


        for y in 0..4 {
            print!("│");
            for _ in 0..((SCREEN_WIDTH / 4) - 9) {
                print!(" ");
            }
            for x in 0..4 {
                let key = KEY_ORDER[y * 4 + x];
                let pressed = keypad[KEY_ORDER_HEX[y * 4 + x]];

                print!("│");
                if pressed {
                    print!("\x1b[7m");
                }

                print!(" {} ", key);

                if pressed {
                    print!("\x1b[0m");
                }

                print!("│");
            }
            for _ in 0..((SCREEN_WIDTH / 4) - 9) {
                print!(" ");
            }
            println!("│\r");

            // Draw the middle border
            print!("│");
            for _ in 0..((SCREEN_WIDTH / 4) - 9) {
                print!(" ");
            }
            if y < 3 {
                print!("├───┤├───┤├───┤├───┤");
            } else {
                print!("╰───╯╰───╯╰───╯╰───╯");
            }
            for _ in 0..((SCREEN_WIDTH / 4) - 9) {
                print!(" ");
            }
            println!("│\r");
        }
    }

    // Spacing
    print!("│");
    for _ in 0..((SCREEN_WIDTH / 2) + 2) {
        print!(" ");
    }
    println!("│\r");

    // Draw the outside border
    print!("╰");
    for _ in 0..((SCREEN_WIDTH / 2) + 2) {
        print!("─");
    }
    println!("╯\r");
}

/// Map a key to a button
/// 
/// ## Arguments
/// 
/// * `key` - The key to map
/// 
/// ## Returns
/// 
/// The button that the key maps to, or None if the key does not map to a button
fn map_key_to_button(key: KeyCode) -> Option<usize> {
    return match key {
        KeyCode::Char('1') => Some(0x1),
        KeyCode::Char('2') => Some(0x2),
        KeyCode::Char('3') => Some(0x3),
        KeyCode::Char('4') => Some(0xC),
        KeyCode::Char('q') => Some(0x4),
        KeyCode::Char('w') => Some(0x5),
        KeyCode::Char('e') => Some(0x6),
        KeyCode::Char('r') => Some(0xD),
        KeyCode::Char('a') => Some(0x7),
        KeyCode::Char('s') => Some(0x8),
        KeyCode::Char('d') => Some(0x9),
        KeyCode::Char('f') => Some(0xE),
        KeyCode::Char('z') => Some(0xA),
        KeyCode::Char('x') => Some(0x0),
        KeyCode::Char('c') => Some(0xB),
        KeyCode::Char('v') => Some(0xF),
        _ => None,
    };
}
