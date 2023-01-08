pub mod error;
pub mod instruction;
pub mod processes;
pub mod processor;
pub mod state;
pub mod utils;

use solana_program::{
    entrypoint,
};
use crate::processor::process_instruction;

entrypoint!(process_instruction);

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
