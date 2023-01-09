use crate::{
    instruction::{vote_create_account, vote_initialize_account},
    log,
    state::{constants::*, GeneralData, ValidatorConfig, VoteInit},
    utils::{
        get_clock_data_from_account, get_rent_data_from_account, AccountInfoHelpers, ResultExt,
    },
};

use borsh::BorshSerialize;

use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    stake::{
        self,
        state::{Authorized, Lockup, StakeState},
    },
    system_instruction,
    sysvar::{self},
};
use spl_associated_token_account::*;
///Creates the vote account for the selected validator.
/// Also Creates the Stake account for all the primary stake for the validator.
pub fn create_vote_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
    clock_is_from_account: bool,
) -> ProgramResult {
    log!(log_level, 4, "Creating vote account ...");
    let account_info_iter = &mut accounts.iter();

    let validator_info = next_account_info(account_info_iter)?;
    let vote_account_info = next_account_info(account_info_iter)?;
    let sysvar_rent_info = next_account_info(account_info_iter)?;
    let sysvar_clock_info = next_account_info(account_info_iter)?;
    let spl_token_program_account_info = next_account_info(account_info_iter)?;
    let stake_account_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;

    log!(log_level, 0, "Done collecting main accounts");

    sysvar_clock_info
        .assert_key_match(&sysvar::clock::id())
        .error_log("sent sysvar_clock_address is dissimilar from expected one")?;
    sysvar_rent_info
        .assert_key_match(&sysvar::rent::id())
        .error_log("sent sysvar_rent_address is dissimilar from expected one")?;
    spl_token_program_account_info
        .assert_key_match(&spl_token::id())
        .error_log("sent spl_token_program_address is dissimilar from expected one")?;
    config_account_info
        .assert_owner(program_id)
        .error_log("Error @ config_account ownership assertion")?;
    config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("Error @ config_account seed assertion")?;
    general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])
        .error_log("Error @ general_account seed assertion")?;

    let config_data = Box::new(ValidatorConfig::decode(config_account_info))
        .error_log("Error @ config_account data decoding")?;
    let mut general_data = Box::new(GeneralData::decode(general_account_info))
        .error_log("Error @ general_account data decoding")?;

    let (expected_vote_pubkey, expected_vote_pubkey_bump) = vote_account_info
        .assert_seed(program_id, &[VOTE_ACCOUNT_KEY.as_ref()])
        .error_log("failed to assert pda input to vote_account_info")?;
    let (authorized_withdrawer, _authorized_withdrawer_nonce) =
        Pubkey::find_program_address(&[AUTHORIZED_WITHDRAWER_KEY.as_ref()], program_id);

    let (_expected_stake_key, expected_stake_bump) = stake_account_info
        .assert_seed(program_id, &[STAKE_ACCOUNT_KEY.as_ref()])
        .error_log("failed to assert pda input to stake_account_info")?;
    
    let (pd_pool_account_key, _pd_bump) = Pubkey::find_program_address(&[PD_POOL_ACCOUNT_KEY.as_ref()], program_id);

    log!(log_level, 0, "Done with main accounts assertions");
    let clock_data = get_clock_data_from_account(sysvar_clock_info)
        .error_log("failed to get clock data from sysvar_clock_info")?;
    let rent_data = get_rent_data_from_account(sysvar_rent_info)
        .error_log("failed to get rent data from sysvar_rent_info")?;

    general_data.last_withdraw_epoch = clock_data.epoch;

    let vote_init = VoteInit {
        node_pubkey: *validator_info.key,
        authorized_voter: *validator_info.key,
        commission: config_data.commission,
        authorized_withdrawer,
    };
    log!(log_level, 2, "Creating vote_account @vote_program invoke");
    if !clock_is_from_account {
        invoke_signed(
            &vote_create_account(validator_info.key, vote_account_info.key),
            &[validator_info.clone(), vote_account_info.clone()],
            &[&[VOTE_ACCOUNT_KEY.as_ref(), &[expected_vote_pubkey_bump]]],
        )
        .error_log("failed to create vote_account @vote_program invoke")?;
    }
    log!(
        log_level,
        2,
        "Done creating vote_account @vote_program invoke"
    );

    log!(
        log_level,
        2,
        "Initializing vote_account @vote_program invoke"
    );
    invoke(
        &vote_initialize_account(vote_account_info.key, &vote_init),
        &[
            vote_account_info.clone(),
            sysvar_rent_info.clone(),
            sysvar_clock_info.clone(),
            validator_info.clone(),
        ],
    )
    .error_log("failed to initialize vote_account @vote_program invoke")?;
    log!(
        log_level,
        2,
        "Done creating vote_account @vote_program invoke"
    );

    let authorized = &Authorized {
        staker: pd_pool_account_key,
        withdrawer: pd_pool_account_key,
    };
    let lockup = &Lockup {
        unix_timestamp: 0,
        epoch: 0,
        custodian: pd_pool_account_key,
    };

    let lamports =
        LAMPORTS_PER_SOL + rent_data.minimum_balance(std::mem::size_of::<StakeState>() as usize);
    log!(
        log_level,
        2,
        "Creating stake_account @system_program invoke"
    );
    invoke_signed(
        &system_instruction::create_account(
            validator_info.key,
            stake_account_info.key,
            lamports,
            std::mem::size_of::<StakeState>() as u64,
            &stake::program::id(),
        ),
        &[validator_info.clone(), stake_account_info.clone()],
        &[&[
            STAKE_ACCOUNT_KEY.as_ref(),
            expected_vote_pubkey.as_ref(),
            &[expected_stake_bump],
        ]],
    )
    .error_log("failed to create stake_account @system_program invoke")?;
    log!(
        log_level,
        2,
        "Done creating stake_account @system_program invoke"
    );

    general_data.last_total_staked =
        LAMPORTS_PER_SOL + rent_data.minimum_balance(std::mem::size_of::<StakeState>() as usize);

    log!(log_level, 2, "Initializing stake");
    invoke(
        &solana_program::stake::instruction::initialize(stake_account_info.key, authorized, lockup),
        &[stake_account_info.clone(), sysvar_rent_info.clone()],
    )
    .error_log("failed to initialize stake_account @stake_program invoke")?;

    general_data
        .serialize(&mut &mut general_account_info.data.borrow_mut()[..])
        .error_log("Error @ general_account data serialization")?;

    log!(log_level, 4, "Vote account created !!!");

    Ok(())
}
