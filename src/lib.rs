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