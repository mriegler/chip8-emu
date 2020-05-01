use std::{thread, time, error::Error};
use hex::decode;
use crossterm::{
    execute,
    ExecutableCommand,
    QueueableCommand,
    cursor,
    queue,
    terminal
};
use std::io::{stdout, Write};

struct State {
    memory: [u8; 4096],
    registers: [u8; 16],
    address_register: u16,
    counter: u16,
    stack: Vec<u8>,
    current_op_index: u16,
    delay_timer: u8,
    sound_timer: u8,
    pixels: [[bool; 32]; 64],
    keys: u16
}

impl Default for State {
    fn default() -> Self {
        State {
            memory: [0u8; 4096],
            registers: [0u8; 16],
            address_register: 0,
            counter: 0,
            stack: Vec::new(),
            current_op_index: 512,
            delay_timer: 0,
            sound_timer: 0,
            pixels: [[false; 32]; 64],
            keys: 0
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut state: State = Default::default();
    let mut code = String::new();

    code.push_str("611E"); //set v1 to 30 . 512
    code.push_str("620E"); //set v2 to 14 . 514
    code.push_str("A21C"); // sprite is at 540  . 516
    code.push_str("00E0"); //clear screen . 518
    code.push_str("0000"); //             . 520
    code.push_str("D124"); // draw x = v1, y = v2, 4 bytes  . 522
    code.push_str("0000"); //             . 524
    code.push_str("0000"); //             . 526
    code.push_str("120C"); //  jmp to 512 . 528
    code.push_str("0000"); //             . 530
    code.push_str("0000"); //             . 532
    code.push_str("0000"); //             . 534
    code.push_str("0000"); //             . 536
    code.push_str("0000"); //             . 538
    code.push_str("183C"); //             . 540
    code.push_str("7EFF"); //             . 542
    code.push_str("0000"); //             . 544

    stdout().execute(terminal::Clear(terminal::ClearType::All))?;


    load_program(&mut state.memory, &code);
    loop {
        let op = get_op_at(&state.memory, state.current_op_index);
        let mut next_op_index = state.current_op_index + 2;
        let op1 = (op >> 12) as u8;
        let op2 = (op >> 8 & 15) as u8;
        let op3 = (op >> 4 & 15) as u8;
        let op4 = (op & 15) as u8;
        
        if op1 == 1 {
            //jump
            next_op_index = op & 4095;
        } else if op == 14 * 16 {
            //clear screen
            for i in 0..state.pixels.len() {
                for j in 0..32 {
                    state.pixels[i][j] = false;
                }
            }
        } else if op1 == 10 {
            // set address register to the rest
            state.address_register = op & 4095;
        } else if op1 == 6 {
            // set register
            state.registers[(op2 as usize)] = op as u8;
        } else if op1 == 13 {
            // render sprite
            let x = state.registers[op2 as usize] as usize;
            let y = state.registers[op3 as usize] as usize;
            let source_start = state.address_register as usize;
            let height = op4 as usize;

            println!("x: {}\ny: {}\nsource_start: {}\nheight: {}", x, y, source_start, height);
            
            let mut collided = false;
            for i in 0..height {
                let source = &state.memory[source_start + i];
                println!("cur src {:?}", source);
                for j in 0..8 {
                    let source_bit = source >> (7 - j) & 0b1;
                    let source_bool = source_bit > 0;
                    let target_pixel = &mut state.pixels[x + j][y + i];

                    let new_pixel = source_bool ^ *target_pixel;
                    if new_pixel != *target_pixel {
                        collided = true;
                    }
                    println!("targetPixel {},sourceBool: {},j {},newPixel {}", target_pixel, source_bool, j, new_pixel);
                    *target_pixel = new_pixel;
                }
            }

            state.registers[15] = collided as u8;
        }

        render_pixels(&state.pixels)?;

        state.current_op_index = next_op_index;
        thread::sleep(time::Duration::from_millis(20));
    }
}

fn render_pixels(pixels: &[[bool; 32]; 64]) -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();
    stdout.queue(cursor::MoveTo(0,0))?;
    for y in 0..32 {
        for x in 0..64 {
            let pixel = if pixels[x][y] { "X" } else { "-" };
            stdout.write(format!("{}", pixel).as_bytes())?;
        }
        stdout.queue(cursor::MoveToNextLine(1))?;
    }
    stdout.flush()?;
    return Ok(());
}

fn get_op_at(memory: &[u8; 4096], index: u16) -> u16 {
    let index = index as usize;
    if index > 4094 {
        panic!("getting op outside of memory");
    }

    return (memory[index] as u16) << 8 | memory[index + 1] as u16;
}

fn load_program(memory: &mut [u8; 4096], program: &str) {
    let program = match decode(program) {
        Ok(decoded) => decoded,
        Err(error) => panic!("cant decode program {:?}", error)
    };

    let sliced: &mut [u8] = &mut memory[512..];

    for (i, byte) in sliced.iter_mut().enumerate() {
        if i < program.len() {
            *byte = program[i];
        } else {
            break;
        }
    }
}

