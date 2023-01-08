
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

pub fn process_instruction(_program_id: &Pubkey, _accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    msg!("Hello, world!");
    Ok(())
}