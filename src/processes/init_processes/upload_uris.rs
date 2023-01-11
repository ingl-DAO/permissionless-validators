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
    program::invoke,
    pubkey::Pubkey,
    system_instruction,
};

pub fn upload_uris(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    uris: Vec<String>,
    rarity: u8,
    log_level: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let team_account_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;
    let uris_account_info = next_account_info(account_info_iter)?;

    uris_account_info
        .assert_owner(program_id)
        .error_log("Error: uris_account is not owned by the program")?;
    uris_account_info
        .assert_seed(program_id, &[URIS_ACCOUNT_SEED.as_ref()])
        .error_log("Error: uris_account is not the config account")?;
    config_account_info
        .assert_owner(program_id)
        .error_log("Error: Config account is not owned by the program")?;
    config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("Error: Config account is not the config account")?;
    team_account_info
        .assert_signer()
        .error_log("Error: Team account is not a signer")?;

    let config = Box::new(ValidatorConfig::decode(config_account_info)?);

    team_account_info
        .assert_key_match(&config.validator_id)
        .error_log("Error: Team account is not the team account")?;

    let mut uris_account_data = Box::new(UrisAccount::decode(uris_account_info)?);
    log!(
        log_level,
        0,
        "Uploading URIs {:?} for rarity {}",
        uris,
        rarity,
    );
    let incremented_space = uris_account_data
        .set_uri(rarity, uris)
        .error_log("Error @ uri setting")?;
    log!(log_level, 2, "Uploaded URIs for generation !!");

    let space = uris_account_info.data_len() + incremented_space;
    let lamports = Rent::get()?.minimum_balance(space)
        - Rent::get()?.minimum_balance(uris_account_info.data_len());
    log!(
        log_level,
        2,
        "Adding {} lamports to config account",
        lamports
    );
    invoke(
        &system_instruction::transfer(team_account_info.key, uris_account_info.key, lamports),
        &[team_account_info.clone(), uris_account_info.clone()],
    )
    .error_log("Error: Failed to transfer lamports to config account")?;
    log!(
        log_level,
        2,
        "Adding {} bytes to config account",
        incremented_space
    );
    uris_account_info
        .realloc(space, false)
        .error_log("Error: Failed to realloc config account")?;
    uris_account_data
        .serialize(&mut &mut uris_account_info.data.borrow_mut()[..])
        .error_log("Error: Failed to serialize config")?;
    Ok(())
}
