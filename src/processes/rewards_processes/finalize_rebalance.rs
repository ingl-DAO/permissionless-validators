use crate::{
    error::InglError,
    log,
    state::{constants::*, GeneralData, ValidatorConfig},
    utils::{AccountInfoHelpers, ResultExt},
};

use borsh::BorshSerialize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    pubkey::Pubkey,
    sysvar::{self},
};

pub fn finalize_rebalance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
) -> ProgramResult {
    log!(log_level, 4, "Initiated Finalizing rebalance ...");
    let account_info_iter = &mut accounts.iter();
    let _payer_account_info = next_account_info(account_info_iter)?;
    let validator_account_info = next_account_info(account_info_iter)?;
    let t_stake_account_info = next_account_info(account_info_iter)?;
    let pd_pool_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;
    let sysvar_clock_info = next_account_info(account_info_iter)?;
    let stake_account_info = next_account_info(account_info_iter)?;
    let t_withdraw_info = next_account_info(account_info_iter)?;
    let sysvar_stake_history_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;

    log!(log_level, 0, "Done with account collection");

    let (_pd_pool_pubkey, pd_pool_bump) = pd_pool_account_info
        .assert_seed(program_id, &[PD_POOL_ACCOUNT_KEY.as_ref()])
        .error_log("failed to assert pd_pool pubkey")?;

    let (_expected_t_stake_key, _expected_t_stake_bump) = t_stake_account_info
        .assert_seed(program_id, &[T_STAKE_ACCOUNT_KEY.as_ref()])
        .error_log("failed to assert t_stake pubkey")?;

    let (_general_account_key, _general_account_bump) = general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])
        .error_log("failed to assert vote data pubkey")?;
    let (_config_account_key, _config_account_bump) = config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("failed to assert config data pubkey")?;
    general_account_info
        .assert_owner(program_id)
        .error_log("failed to assert ingl_vote_data_account ownership")?;
    config_account_info
        .assert_owner(program_id)
        .error_log("failed to assert config account ownership")?;
    let mut general_data = Box::new(GeneralData::parse(general_account_info, program_id)?);
    let config_data = Box::new(ValidatorConfig::parse(config_account_info, program_id)?);

    sysvar_stake_history_info
        .assert_key_match(&sysvar::stake_history::id())
        .error_log("failed to assert stake history pubkey")?;
    sysvar_clock_info
        .assert_key_match(&sysvar::clock::id())
        .error_log("failed to assert clock pubkey")?;

    validator_account_info
        .assert_key_match(&config_data.validator_id)
        .error_log("failed to assert validator pubkey")?;

    sysvar_clock_info
        .assert_owner(&sysvar::id())
        .error_log("failed to assert clock account ownership")?;
    sysvar_stake_history_info
        .assert_owner(&sysvar::id())
        .error_log("failed to assert stake history account ownership")?;
    stake_account_info
        .assert_owner(&solana_program::stake::program::id())
        .error_log("failed to assert stake account ownership")?;

    let (_expected_stake_key, _expected_stake_bump) = stake_account_info
        .assert_seed(program_id, &[STAKE_ACCOUNT_KEY.as_ref()])
        .error_log("failed to assert stake pubkey")?;

    let (_expected_t_withdraw_key, _t_withdraw_bump) = t_withdraw_info
        .assert_seed(program_id, &[T_WITHDRAW_KEY.as_ref()])
        .error_log("failed to assert t_withdraw pubkey")?;

    log!(log_level, 0, "Done with account assertions");

    if general_data.is_t_stake_initialized {
        t_stake_account_info
            .assert_owner(&solana_program::stake::program::id())
            .error_log("failed to assert t_stake account ownership")?;
        log!(
            log_level,
            2,
            "Merging t stake account into stake account ..."
        );
        invoke_signed(
            &solana_program::stake::instruction::merge(
                stake_account_info.key,
                t_stake_account_info.key,
                pd_pool_account_info.key,
            )[0],
            &[
                stake_account_info.clone(),
                t_stake_account_info.clone(),
                sysvar_clock_info.clone(),
                sysvar_stake_history_info.clone(),
                pd_pool_account_info.clone(),
            ],
            &[&[PD_POOL_ACCOUNT_KEY.as_ref(), &[pd_pool_bump]]],
        )
        .error_log("failed to merge t stake account into stake account")?;
        log!(
            log_level,
            2,
            "Merged t stake account into stake account !!!"
        );
    }
    if !general_data.rebalancing_data.is_rebalancing_active {
        Err(InglError::TooEarly.utilize("init rebalance not active"))?
    }

    if general_data.rebalancing_data.unclaimed_validator_rewards == 0
        && t_withdraw_info.owner == &solana_program::stake::program::id()
    {
        log!(
            log_level,
            2,
            "Withdrawing from t_withdraw account to pd_pool_account_info ..."
        );
        invoke_signed(
            &solana_program::stake::instruction::withdraw(
                t_withdraw_info.key,
                pd_pool_account_info.key,
                pd_pool_account_info.key,
                t_withdraw_info.lamports(),
                None,
            ),
            &[
                t_withdraw_info.clone(),
                pd_pool_account_info.clone(),
                sysvar_clock_info.clone(),
                sysvar_stake_history_info.clone(),
                pd_pool_account_info.clone(),
            ],
            &[&[PD_POOL_ACCOUNT_KEY.as_ref(), &[pd_pool_bump]]],
        )
        .error_log("failed to withdraw from t_withdraw account to pd_pool_account_info")?;
        log!(
            log_level,
            2,
            "Withdrew from t_withdraw account to pd_pool_account_info !!!"
        );
    }
    if general_data.rebalancing_data.pending_validator_rewards > 0 {
        log!(
            log_level,
            2,
            "Withdrawing from validator_rewards from pd_pool_acount to validator id ..."
        );
        invoke_signed(
            &solana_program::system_instruction::transfer(
                pd_pool_account_info.key,
                validator_account_info.key,
                general_data.rebalancing_data.pending_validator_rewards,
            ),
            &[pd_pool_account_info.clone(), validator_account_info.clone()],
            &[&[PD_POOL_ACCOUNT_KEY.as_ref(), &[pd_pool_bump]]],
        )
        .error_log("failed to withdraw from pd_pool_account to validator_id")?;
        log!(log_level, 2, "Withdrew from pd_pool_ !!!");
    }

    general_data.rebalancing_data.pending_validator_rewards = 0;

    general_data.rebalancing_data.is_rebalancing_active = false;

    log!(log_level, 0, "Serializing general_account_data ...");
    general_data
        .serialize(&mut &mut general_account_info.data.borrow_mut()[..])
        .error_log("failed to serialize into general account info")?;

    log!(log_level, 4, "finished finalize_rebalance!!!");
    Ok(())
}
