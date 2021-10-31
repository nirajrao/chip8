#[derive(Debug)]
/// Helper struct to make data fetching easier in chip.rs.
pub struct Opcode {
    pub value: u16,
}

impl Opcode {
    pub fn fetch_highest_nibble(&self) -> u16 {
        self.value & 0xF000
    }

    pub fn fetch_lowest_nibble(&self) -> u16 {
        self.value & 0x000F
    }

    pub fn fetch_lowest_byte(&self) -> u8 {
        (self.value & 0x00FF).to_be_bytes()[1]
    }

    pub fn fetch_nnn(&self) -> u16 {
        // NNN - 12 lowest bits of instruction. Represents an address.
        self.value & 0x0FFF
    }

    pub fn fetch_x(&self) -> usize {
        // X - 4 lowest bits of high byte of instruction.
        usize::from((self.value & 0x0F00).to_be_bytes()[0])
    }

    pub fn fetch_y(&self) -> usize {
        // Y - 4 highest bits of low byte of instruction.
        usize::from((self.value & 0x00F0).to_be_bytes()[1] >> 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_highest_nibble() {
        let opcode = Opcode { value: 0x1234 };
        let highest_nibble = opcode.fetch_highest_nibble();
        assert_eq!(highest_nibble, 0x1000);
    }

    #[test]
    fn test_fetch_lowest_nibble() {
        let opcode = Opcode { value: 0x1234 };
        let lowest_nibble = opcode.fetch_lowest_nibble();
        assert_eq!(lowest_nibble, 0x0004);
    }

    #[test]
    fn test_fetch_lowest_byte() {
        let opcode = Opcode { value: 0x1234 };
        let lowest_nibble = opcode.fetch_lowest_byte();
        assert_eq!(lowest_nibble, 0x0034);
    }

    #[test]
    fn test_fetch_nnn() {
        let opcode = Opcode { value: 0x1234 };
        let lowest_nibble = opcode.fetch_nnn();
        assert_eq!(lowest_nibble, 0x0234);
    }

    #[test]
    fn test_fetch_x() {
        let opcode = Opcode { value: 0x1234 };
        let lowest_nibble = opcode.fetch_x();
        assert_eq!(lowest_nibble, 0x2);
    }

    #[test]
    fn test_fetch_y() {
        let opcode = Opcode { value: 0x1234 };
        let lowest_nibble = opcode.fetch_y();
        assert_eq!(lowest_nibble, 0x3);
    }
}
