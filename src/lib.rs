pub mod error;
pub mod instruction;
pub mod processes;
pub mod processor;
pub mod state;
pub mod utils;

use crate::processor::process_instruction;
use solana_program::entrypoint;

entrypoint!(process_instruction);

#[cfg(test)]
pub mod tests {
    pub fn add(number1: u64, number2: u64) -> u64 {
        return number1 + number2;
    }

    #[test]
    pub fn custom_test() {
        assert_eq!(add(1430, 1780), 3210);
    }
}
