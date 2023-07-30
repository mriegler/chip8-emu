use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::{cursor, style::Colorize, terminal, ExecutableCommand, QueueableCommand};
use log::log;
use rand::Rng;
use state::State;
use std::io::stdout;
use std::{env, error::Error, fs, process, thread, time};
pub mod log;
pub mod screen;
pub mod state;
pub mod util;

const FONT: &'static [u8; 80] = &[
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const KEY_MAP: &'static [KeyCode; 16] = &[
    KeyCode::Char('0'),
    KeyCode::Char('1'),
    KeyCode::Char('2'),
    KeyCode::Char('3'),
    KeyCode::Char('4'),
    KeyCode::Char('5'),
    KeyCode::Char('6'),
    KeyCode::Char('7'),
    KeyCode::Char('8'),
    KeyCode::Char('9'),
    KeyCode::Char('/'),
    KeyCode::Char('*'),
    KeyCode::Char('-'),
    KeyCode::Char('+'),
    KeyCode::Enter,
    KeyCode::Char('.'),
];

fn main() -> Result<(), Box<dyn Error>> {
    let mut state: State = Default::default();
    let mut rng = rand::thread_rng();
    log("asd");
    log(format!("begin"));

    if let Some(path) = env::args().nth(1) {
        log(format!("path {}", path));
        if let Ok(contents) = &fs::read(path) {
            load_program_bytes(&mut state.memory[512..], contents);
        } else {
            panic!("couldnt read file");
        }
    } else {
        panic!("provide program to load as first arg");
    }

    load_program_bytes(&mut state.memory, FONT);

    stdout().execute(terminal::Clear(terminal::ClearType::All))?;
    terminal::enable_raw_mode()?;
    loop {
        let start_time = time::Instant::now();
        let op = get_op_at(&state.memory, state.current_op_index);
        let mut next_op_index = state.current_op_index + 2;
        let op1 = (op >> 12) as u8;
        let op2 = (op >> 8 & 15) as u8;
        let op3 = (op >> 4 & 15) as u8;
        let op4 = (op & 15) as u8;

        log(format!("op {:#x}", op));

        if state.delay_timer > 0 {
            state.delay_timer -= 1;
        }
        if state.sound_timer > 0 {
            state.sound_timer -= 1;
        }

        if poll(time::Duration::from_millis(2)).unwrap() {
            match read().unwrap() {
                // q for quit
                Event::Key(key_event) if key_event.code == KeyCode::Char('q') => {
                    terminal::disable_raw_mode()?;
                    process::exit(0);
                }
                Event::Key(key_event) => {
                    // detection on term seems fucky, especially for multiple keys
                    match KEY_MAP.iter().position(|&x| x == key_event.code) {
                        Some(i) => state.key = i as u8,
                        _ => (),
                    }
                }
                _ => {
                    state.key = 0;
                }
            };
        } else {
            state.key = 0;
        }
        if op1 == 1 {
            //jump
            log(format!("jumping to {}", op & 4095));
            next_op_index = op & 4095;
        } else if op == 14 * 16 + 14 {
            //return from subroutine
            let ret = state
                .stack
                .pop()
                .expect("tried returning from sub with an empty stack");
            next_op_index = ret;
            log(format!("returning from sub to {}", next_op_index));
        } else if op1 == 2 {
            // execute subroutine
            state.stack.push(next_op_index);
            next_op_index = op & 4095;
            log(format!("run subroutine at {}", next_op_index));
        } else if op1 == 3 {
            //if reg x == NN skip next op
            log(format!("check if reg {} == {}", op2 as usize, op as u8));
            if state.registers[op2 as usize] == op as u8 {
                next_op_index += 2;
            }
        } else if op1 == 4 {
            // if reg x != NN skip next op
            log(format!("checking if reg {} != {}", op2, op as u8));
            if state.registers[op2 as usize] != op as u8 {
                log(format!("skipping next"));
                next_op_index += 2;
            }
        } else if op1 == 5 {
            // if reg x == reg y skip next op
            if state.registers[op2 as usize] == state.registers[op3 as usize] {
                next_op_index += 2;
            }
        } else if op == 14 * 16 {
            //clear screen
            log(format!("clear screen"));
            state.screen.clear_screen();
        } else if op1 == 10 {
            // set address register to the rest
            state.address_register = op & 4095;
            log(format!("set addr reg to {}", state.address_register));
        } else if op1 == 6 {
            // set register
            log(format!("set register {} to {}", op2, op as u8));
            state.registers[op2 as usize] = op as u8;
        } else if op1 == 7 {
            // add to reg x NN
            let reg = &mut state.registers[op2 as usize];
            *reg = reg.wrapping_add(op as u8);
        } else if op1 == 8 && op4 == 0 {
            //assign reg x to reg y
            state.registers[op2 as usize] = state.registers[op3 as usize];
        } else if op1 == 8 && op4 == 1 {
            //set reg x to reg x | reg y
            let reg_y = state.registers[op3 as usize];
            let reg_x = &mut state.registers[op2 as usize];
            *reg_x |= reg_y;
        } else if op1 == 8 && op4 == 2 {
            //set reg x to reg x & reg y
            let reg_y = state.registers[op3 as usize];
            let reg_x = &mut state.registers[op2 as usize];
            *reg_x &= reg_y;
        } else if op1 == 8 && op4 == 3 {
            //set reg x to reg x ^ reg y
            let reg_y = state.registers[op3 as usize];
            let reg_x = &mut state.registers[op2 as usize];
            *reg_x ^= reg_y;
        } else if op1 == 8 && op4 == 4 {
            // set reg x to reg x + reg y
            let reg_y = state.registers[op3 as usize];
            let reg_x_old = state.registers[op2 as usize];
            let reg_x = &mut state.registers[op2 as usize];
            *reg_x = reg_x.wrapping_add(reg_y);

            // carry over?
            state.registers[15] = (reg_x_old > *reg_x) as u8;
        } else if op1 == 8 && op4 == 5 {
            // set reg x to reg x - reg y
            let reg_y = state.registers[op3 as usize];
            let reg_x_old = state.registers[op2 as usize];
            let reg_x = &mut state.registers[op2 as usize];
            *reg_x = reg_x.wrapping_sub(reg_y);

            // borrow?
            state.registers[15] = (reg_x_old >= *reg_x) as u8;
        } else if op1 == 8 && op4 == 6 {
            // shift reg x right
            let reg_x = &mut state.registers[op2 as usize];
            let dropped_bit = *reg_x & 1;
            *reg_x = *reg_x >> 1;
            state.registers[15] = dropped_bit;
        } else if op1 == 8 && op4 == 7 {
            //set reg x to reg y - reg x
            let reg_y = state.registers[op3 as usize];
            let reg_x = &mut state.registers[op2 as usize];
            *reg_x = reg_y - *reg_x;

            // borrow?
            state.registers[15] = (reg_y >= *reg_x) as u8;
        } else if op1 == 8 && op4 == 14 {
            // shift reg x left
            let reg_x = &mut state.registers[op2 as usize];
            let dropped_bit = *reg_x & 128;
            *reg_x = *reg_x << 1;
            state.registers[15] = dropped_bit;
        } else if op1 == 9 {
            // skip next if reg x != reg y
            log(format!("checking if reg {} != reg {}", op2, op3));
            if state.registers[op2 as usize] != state.registers[op3 as usize] {
                log(format!("skipping next"));
                next_op_index += 2;
            }
        } else if op1 == 11 {
            // jump to NNN + reg 0
            let reg_0 = state.registers[0];
            next_op_index = reg_0 as u16 + (op & 4095);
        } else if op1 == 12 {
            // set reg x to NN & random()
            let reg_x = &mut state.registers[op2 as usize];
            *reg_x = (op & 255) as u8 & rng.gen_range(0, 256 as u16) as u8;
        } else if op1 == 13 {
            // render sprite
            let x = state.registers[op2 as usize] as usize;
            let y = state.registers[op3 as usize] as usize;
            let source_start = state.address_register as usize;
            let height = op4 as usize;
            let collided = state.screen.render_sprite(
                x,
                y,
                &state.memory[source_start..(source_start + height)],
            );

            log(format!(
                "rendering sprite,x {} y {}, height {}",
                x, y, height
            ));
            state.registers[15] = collided as u8;
        } else if op1 == 14 && op4 == 14 {
            // skip next if button in reg x is pressed
            let reg = state.registers[op2 as usize];
            if state.key == reg {
                next_op_index += 2;
            }
        } else if op1 == 14 && op4 == 1 {
            // skip next if button in reg x is not pressed
            let reg = state.registers[op2 as usize];
            log(format!(
                "check if button {} is not pressed, {}",
                reg, state.key
            ));
            if state.key != reg {
                log(format!("skipping bc button {} not pressed", reg));
                next_op_index += 2;
            }
        } else if op1 == 15 && op4 == 7 {
            // set reg x to delay timer
            state.registers[op2 as usize] = state.delay_timer;
            log(format!(
                "set reg {} to delay, val {}",
                op2, state.delay_timer
            ));
        } else if op1 == 15 && op4 == 10 {
            // wait for key
            log(format!("waiting for keypress"));
            let key = wait_for_key();
            state.registers[op2 as usize] = key;
        } else if op1 == 15 && op3 == 1 && op4 == 5 {
            // set delay timer to reg x
            let reg = state.registers[op2 as usize];
            state.delay_timer = reg;
            log(format!("set delay timer to reg {}, val {}", op2, reg));
        } else if op1 == 15 && op3 == 1 && op4 == 8 {
            // set sound timer to reg x
            let reg = state.registers[op2 as usize];
            state.sound_timer = reg;
        } else if op1 == 15 && op3 == 1 && op4 == 14 {
            // add reg x to address register
            let reg = state.registers[op2 as usize];
            let old_address_register = state.address_register;
            state.address_register = state.address_register.wrapping_add(reg as u16);
            log(format!(
                "add {} to addr reg (old {}, new {})",
                reg, old_address_register, state.address_register
            ));
            // overflow?
            state.registers[15] = (old_address_register > state.address_register) as u8;
        } else if op1 == 15 && op3 == 2 && op4 == 9 {
            // set address_register to font address for digit in reg x
            log(format!(
                "setting address register to font {}",
                state.registers[op2 as usize]
            ));
            state.address_register = (state.registers[op2 as usize] * 5) as u16;
        } else if op1 == 15 && op3 == 3 && op4 == 3 {
            // store binary coded decimal of reg x at adress_register
            let reg = state.registers[op2 as usize];
            let hundreds = reg / 100;
            let tens = (reg / 10) % 10;
            let ones = reg % 10;

            log(format!(
                "storing bcd of {}: {} {} {}",
                reg, hundreds, tens, ones
            ));
            state.memory[state.address_register as usize] = hundreds;
            state.memory[(state.address_register as usize) + 1] = tens;
            state.memory[(state.address_register as usize) + 2] = ones;
        } else if op1 == 15 && op3 == 5 && op4 == 5 {
            // dump regsiters up to x inclusive to memory at address_register
            for i in 0..op2 {
                let reg = state.registers[i as usize];
                state.memory[(state.address_register as usize) + 0] = reg;
            }
        } else if op1 == 15 && op3 == 6 && op4 == 5 {
            // load registers up to x inclusive from mem at address_register
            for i in 0..op2 + 1 {
                let mem_address = (state.address_register as usize) + i as usize;
                let mem = state.memory[mem_address];
                state.registers[i as usize] = mem;
                log(format!(
                    "load reg {} from mem at {}, val {}",
                    i, mem_address, mem
                ));
            }
        }

        handle_sound(&state.sound_timer)?;
        state.screen.render_pixels()?;

        state.current_op_index = next_op_index;
        let time_taken = start_time.elapsed().as_millis();
        log(format!("time taken: {}", time_taken));
        if time_taken < 16 {
            // 60hz tick attempt
            thread::sleep(time::Duration::from_millis(16 - time_taken as u64));
        }
    }
}

fn handle_sound(timer: &u8) -> Result<(), Box<dyn Error>> {
    // sound is too much work, so just visually display sound
    let mut stdout = stdout();
    stdout.queue(cursor::MoveTo(65, 0))?;
    if *timer > 0 {
        print!("{}", "█".magenta());
    } else {
        print!(" ");
    }
    Ok(())
}

fn wait_for_key() -> u8 {
    return match read().unwrap() {
        Event::Key(key_event) => {
            let key_position = KEY_MAP.iter().position(|&r| r == key_event.code);
            match key_position {
                Some(key) => key as u8,
                None => wait_for_key(),
            }
        }
        _ => wait_for_key(),
    };
}

fn get_op_at(memory: &[u8; 4096], index: u16) -> u16 {
    let index = index as usize;
    if index > 4094 {
        panic!("getting op outside of memory");
    }

    return (memory[index] as u16) << 8 | memory[index + 1] as u16;
}

fn load_program_bytes(memory: &mut [u8], program: &[u8]) {
    let sliced: &mut [u8] = &mut memory[0..program.len()];
    sliced.copy_from_slice(program);
}
