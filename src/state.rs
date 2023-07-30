use crate::screen::Screen;

pub struct State {
    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    pub address_register: u16,
    pub stack: Vec<u16>,
    pub current_op_index: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub screen: Screen,
    pub key: u8,
}

impl Default for State {
    fn default() -> Self {
        State {
            memory: [0u8; 4096],
            registers: [0u8; 16],
            address_register: 0,
            stack: Vec::new(),
            current_op_index: 512,
            delay_timer: 0,
            sound_timer: 0,
            screen: Default::default(),
            key: 0,
        }
    }
}

impl State {}
