use crate::{
    log,
    state::{constants::*, UrisAccount, ValidatorConfig},
    utils::{AccountInfoHelpers, ResultExt},
};

use anchor_lang::prelude::{Rent, SolanaSysvar};
use borsh::BorshSerialize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    pubkey::Pubkey,
    system_instruction,
};

pub fn reset_uris(program_id: &Pubkey, accounts: &[AccountInfo], log_level: u8) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;
    let uris_account_info = next_account_info(account_info_iter)?;

    payer_account_info
        .assert_signer()
        .error_log("Error: Payer account is not a signer")?;
    uris_account_info
        .assert_owner(program_id)
        .error_log("Error: uris_account is not owned by the program")?;
    let (_uris_account_key, uris_account_bump) = uris_account_info
        .assert_seed(program_id, &[URIS_ACCOUNT_SEED.as_ref()])
        .error_log("Error: uris_account is not the config account")?;
    config_account_info
        .assert_owner(program_id)
        .error_log("Error: Config account is not owned by the program")?;
    config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("Error: Config account is not the config account")?;

    let config = Box::new(ValidatorConfig::decode(config_account_info)?);
    payer_account_info
        .assert_key_match(&config.validator_id)
        .error_log("Error: Payer account is not the validator_id")?;
    let mut uris_account_data = Box::new(UrisAccount::decode(uris_account_info)?);
    uris_account_data.uris = Vec::new();
    let space = 16;
    let lamports = Rent::get()?.minimum_balance(uris_account_info.data_len())
        - Rent::get()?.minimum_balance(space);
    log!(
        log_level,
        2,
        "Transferring {} lamports to payer account",
        lamports
    );
    invoke_signed(
        &system_instruction::transfer(uris_account_info.key, payer_account_info.key, lamports),
        &[uris_account_info.clone(), payer_account_info.clone()],
        &[&[URIS_ACCOUNT_SEED.as_ref(), &[uris_account_bump]]],
    )
    .error_log("Error: Failed to transfer lamports to payer account")?;
    uris_account_info
        .realloc(space, true)
        .error_log("Error: Failed to realloc uris account")?;
    uris_account_data
        .serialize(&mut &mut uris_account_info.data.borrow_mut()[..])
        .error_log("Error: Failed to serialize into uris account")?;
    Ok(())
}
