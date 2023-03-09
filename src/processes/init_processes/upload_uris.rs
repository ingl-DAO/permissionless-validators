use crate::{
    error::InglError,
    log,
    state::{constants::*, UrisAccount, ValidatorConfig},
    utils::{AccountInfoHelpers, ResultExt},
};

use borsh::BorshSerialize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

pub fn upload_uris(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    uris: Vec<String>,
    rarity: u8,
    log_level: u8,
) -> ProgramResult {
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
    uris_account_info
        .assert_seed(program_id, &[URIS_ACCOUNT_SEED.as_ref()])
        .error_log("Error: uris_account is not the config account")?;
    config_account_info
        .assert_owner(program_id)
        .error_log("Error: Config account is not owned by the program")?;
    config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("Error: Config account is not the config account")?;

    let config = Box::new(ValidatorConfig::parse(config_account_info, program_id)?);
    match payer_account_info.assert_key_match(&config.validator_id) {
        Ok(_) => (),
        Err(_) => payer_account_info.assert_key_match(&team::id()).error_log(
            "Error: Payer account is not the validator_id, or temporarily authorized uploader",
        )?,
    }

    let mut uris_account_data = Box::new(UrisAccount::parse(uris_account_info, program_id)?);
    log!(
        log_level,
        0,
        "Uploading URIs {:?} for rarity {}",
        uris,
        rarity,
    );
    uris_account_data
        .set_uri(rarity, uris)
        .error_log("Error @ uri setting")?;
    log!(log_level, 2, "Uploaded URIs for generation !!");

    let space = uris_account_data.get_space();
    if space > 20000 {
        Err(InglError::UrisAccountTooBig.utilize(
            "Uploaded too many images. Consider Reseting and selecting the best and lesser images",
        ))?
    }
    let lamports: i128 =
        Rent::get()?.minimum_balance(space) as i128 - uris_account_info.lamports() as i128;
    if lamports > 0 {
        log!(
            log_level,
            2,
            "Adding {} lamports to config account",
            lamports
        );
        invoke(
            &system_instruction::transfer(
                payer_account_info.key,
                uris_account_info.key,
                lamports as u64,
            ),
            &[payer_account_info.clone(), uris_account_info.clone()],
        )
        .error_log("Error: Failed to transfer lamports to config account")?;
    }
    log!(
        log_level,
        2,
        "Adding {} bytes to config account",
        space - uris_account_info.data_len()
    );
    uris_account_info
        .realloc(space, false)
        .error_log("Error: Failed to realloc config account")?;
    uris_account_data
        .serialize(&mut &mut uris_account_info.data.borrow_mut()[..])
        .error_log("Error: Failed to serialize config")?;
    Ok(())
}
