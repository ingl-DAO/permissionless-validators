use crate::{
    error::InglError,
    log,
    state::{constants::*, FundsLocation, GeneralData, NftData, ValidatorConfig},
    utils::{get_clock_data, verify_nft_ownership, AccountInfoHelpers, OptionExt, ResultExt},
};

use borsh::BorshSerialize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub fn delegate_gem(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
    clock_is_from_account: bool,
) -> ProgramResult {
    log!(log_level, 4, "Initiated Gem Delegation ...");
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let nft_account_data_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;

    let clock_data = get_clock_data(account_info_iter, clock_is_from_account)?;

    verify_nft_ownership(
        payer_account_info,
        mint_account_info,
        nft_account_data_info,
        associated_token_account_info,
        program_id,
    )?;

    general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])
        .error_log("Error: @general_account_info seed assertion")?;
    config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("Error: @config_account_info seed assertion")?;

    general_account_info
        .assert_owner(program_id)
        .error_log("Error: @general_account_info owner assertion")?;
    config_account_info
        .assert_owner(program_id)
        .error_log("Error: @config_account_info owner assertion")?;

    let mut nft_account_data =
        NftData::parse(&nft_account_data_info, program_id).error_log("Error @ Gem Account Data Validation")?;

    let mut general_account_data = Box::new(GeneralData::parse(general_account_info, program_id)?);
    let config_data = Box::new(ValidatorConfig::parse(config_account_info, program_id)?);

    general_account_data.total_delegated = general_account_data
        .total_delegated
        .checked_add(config_data.unit_backing)
        .error_log("Error @ Global Gem Account Data Delegated Total recalc")?;

    match nft_account_data.funds_location {
        FundsLocation::Undelegated => {
            nft_account_data.funds_location = FundsLocation::Delegated;
            nft_account_data.last_delegation_epoch = Some(clock_data.epoch);
        }
        _ => Err(InglError::InvalidFundsLocation.utilize("gem's funds location."))?,
    }

    if general_account_data.dealloced >= config_data.unit_backing {
        general_account_data.dealloced = general_account_data
            .dealloced
            .checked_sub(config_data.unit_backing)
            .error_log("Error @ General Account Data Dealloced recalc")?;
    } else {
        general_account_data.pending_delegation_total = general_account_data
            .pending_delegation_total
            .checked_add(config_data.unit_backing)
            .error_log("Error @ General Account Data Pending Delegation Total recalc")?;
    }

    if general_account_data.total_delegated > config_data.max_primary_stake {
        Err(InglError::BeyondBounds.utilize("Total stake will Exceed maximum allowed"))?
    }

    nft_account_data
        .serialize(&mut &mut nft_account_data_info.data.borrow_mut()[..])
        .error_log("Error @ Gem Account Data Serialization")?;
    general_account_data
        .serialize(&mut &mut general_account_info.data.borrow_mut()[..])
        .error_log("Error @ General Account Data Serialization")?;

    Ok(())
}
