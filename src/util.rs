pub fn get_bit(source: u8, pos: usize) -> bool {
    source >> pos & 0b1 > 0
}
