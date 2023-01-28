use crate::{
    error::InglError,
    instruction::split,
    log,
    state::{constants::*, GeneralData},
    utils::{get_rent_data_from_account, AccountInfoHelpers, OptionExt, ResultExt},
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

pub fn init_rebalance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
) -> ProgramResult {
    log!(log_level, 4, "initializing init_rebalance ...");
    let account_info_iter = &mut accounts.iter();
    let _payer_account_info = next_account_info(account_info_iter)?;
    let t_stake_account_info = next_account_info(account_info_iter)?;
    let pd_pool_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;
    let sysvar_clock_info = next_account_info(account_info_iter)?;
    let sysvar_rent_info = next_account_info(account_info_iter)?;
    let stake_account_info = next_account_info(account_info_iter)?;
    let t_withdraw_info = next_account_info(account_info_iter)?;
    let vote_account_info = next_account_info(account_info_iter)?;
    let stake_history_account_info = next_account_info(account_info_iter)?;
    let stake_config_account_info = next_account_info(account_info_iter)?;

    log!(log_level, 0, "done with account collection");

    let (pd_pool_pubkey, pd_pool_bump) = pd_pool_account_info
        .assert_seed(program_id, &[PD_POOL_ACCOUNT_KEY.as_ref()])
        .error_log("failed to assert pd_pool_account_info")?;
    let (expected_t_stake_key, expected_t_stake_bump) = t_stake_account_info
        .assert_seed(program_id, &[T_STAKE_ACCOUNT_KEY.as_ref()])
        .error_log("failed to assert t_stake_account_info")?;
    let (_general_account_key, _general_account_bump) = general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])
        .error_log("failed to assert general_account_info")?;
    let (_expected_stake_key, _expected_stake_bump) = stake_account_info
        .assert_seed(program_id, &[STAKE_ACCOUNT_KEY.as_ref()])
        .error_log("failed to assert stake_account_info")?;
    let (_expected_t_withdraw_key, t_withdraw_bump) = t_withdraw_info
        .assert_seed(program_id, &[T_WITHDRAW_KEY.as_ref()])
        .error_log("failed to assert t_withdraw_info")?;
    let (_expected_vote_key, _expected_vote_bump) = vote_account_info
        .assert_seed(program_id, &[VOTE_ACCOUNT_KEY.as_ref()])
        .error_log("failed to assert vote_account_info")?;

    stake_history_account_info.assert_key_match(&solana_program::sysvar::stake_history::id())?;
    stake_config_account_info.assert_key_match(&stake::config::id())?;
    general_account_info
        .assert_owner(program_id)
        .error_log("failed to assert general_account_info program ownership")?;
    t_stake_account_info
        .assert_owner(&solana_program::system_program::id())
        .error_log("Error: @ asserting t_stake_account ownership")?;
    stake_account_info
        .assert_owner(&solana_program::stake::program::id())
        .error_log("Error: @ asserting stake_account ownership")?;
    t_withdraw_info
        .assert_owner(&solana_program::system_program::id())
        .error_log("Error: @ asserting t_withdraw_info ownership")?;
    let mut general_data = Box::new(GeneralData::parse(general_account_info, program_id)?);

    sysvar_clock_info.assert_key_match(&sysvar::clock::id())?;
    sysvar_rent_info.assert_key_match(&sysvar::rent::id())?;
    let rent_data = get_rent_data_from_account(sysvar_rent_info)?;

    log!(log_level, 0, "done with account assertions");

    if general_data.rebalancing_data.is_rebalancing_active {
        Err(InglError::TooLate.utilize("Rebalancing is already ongoing."))?
    }

    let val_owners_lamports = stake_account_info
        .lamports()
        .checked_sub(general_data.last_total_staked)
        .error_log("Stake account has less lamports than expected.")?
        .checked_add(general_data.rebalancing_data.unclaimed_validator_rewards)
        .error_log(
            "Error: @ adding unclaimed_validator_rewards to calculate val_owners_lamports",
        )?;

    let leaving_lamports = val_owners_lamports
        .checked_add(general_data.dealloced)
        .error_log("can't add dealloced to val_owners_lamports")?;
    log!(
        log_level,
        3,
        "leaving_lamports: {}, pending_delegation_lamports: {}",
        leaving_lamports,
        general_data.pending_delegation_total
    );

    if general_data.pending_delegation_total >= leaving_lamports {
        //TODO: since you are creating a stake account here, it must have > LAMPORTS_PER_SOL balance.
        log!(log_level, 3, "leaving_lamports <= Pending delegation Total");
        let lamports = general_data
            .pending_delegation_total
            .checked_sub(leaving_lamports)
            .error_log("Pending delegation total is less than dealloced.")?;

        if lamports
            > LAMPORTS_PER_SOL
                + rent_data.minimum_balance(std::mem::size_of::<StakeState>() as usize)
        {
            log!(log_level, 2, "Creating the Stake account ...");
            invoke_signed(
                &system_instruction::create_account(
                    pd_pool_account_info.key,
                    &expected_t_stake_key,
                    lamports,
                    std::mem::size_of::<StakeState>() as u64,
                    &stake::program::id(),
                ),
                &[pd_pool_account_info.clone(), t_stake_account_info.clone()],
                &[
                    &[PD_POOL_ACCOUNT_KEY.as_ref(), &[pd_pool_bump]],
                    &[T_STAKE_ACCOUNT_KEY.as_ref(), &[expected_t_stake_bump]],
                ],
            )
            .error_log("failed to create the t_stake account")?;
            log!(log_level, 2, "Stake account created!!!");

            let authorized = &Authorized {
                staker: *pd_pool_account_info.key,
                withdrawer: *pd_pool_account_info.key,
            };
            let lockup = &Lockup {
                unix_timestamp: 0,
                epoch: 0,
                custodian: *pd_pool_account_info.key,
            };

            log!(log_level, 2, "Initializing stake ...");
            invoke(
                &solana_program::stake::instruction::initialize(
                    t_stake_account_info.key,
                    authorized,
                    lockup,
                ),
                &[t_stake_account_info.clone(), sysvar_rent_info.clone()],
            )
            .error_log("failed to initialize the t_stake account")?;
            log!(log_level, 2, "Stake initialized!!!");

            log!(log_level, 2, "Delegating stake");
            invoke_signed(
                &solana_program::stake::instruction::delegate_stake(
                    t_stake_account_info.key,
                    pd_pool_account_info.key,
                    vote_account_info.key,
                ),
                &[
                    t_stake_account_info.clone(),
                    vote_account_info.clone(),
                    sysvar_clock_info.clone(),
                    stake_history_account_info.clone(),
                    stake_config_account_info.clone(),
                    pd_pool_account_info.clone(),
                ],
                &[&[PD_POOL_ACCOUNT_KEY.as_ref(), &[pd_pool_bump]]],
            )?;
            log!(log_level, 2, "Done delegating stake");

            general_data.is_t_stake_initialized = true;
            general_data.pending_delegation_total = 0;
            general_data.dealloced = 0;
            general_data.rebalancing_data.pending_validator_rewards = val_owners_lamports;
            general_data.rebalancing_data.unclaimed_validator_rewards = 0;

            general_data.last_total_staked = stake_account_info
                .lamports()
                .checked_add(lamports)
                .error_log("stake_account_info.lamports() + lamports overflows")?;
        } else {
            log!(
                log_level,
                3,
                "Not enough lamports to create a stake account."
            );
            general_data.is_t_stake_initialized = false;
            general_data.rebalancing_data.pending_validator_rewards = val_owners_lamports;
            general_data.rebalancing_data.unclaimed_validator_rewards = 0;
            general_data.pending_delegation_total = general_data.pending_delegation_total.checked_sub(leaving_lamports).error_log("pending_delegation_total in general_data is less than that in leaving_lamports ")?;
            general_data.dealloced = 0;

            general_data.last_total_staked = stake_account_info.lamports();
        }
    } else {
        log!(log_level, 3, "leaving_lamports > Pending delegation Total");
        let split_lamports = leaving_lamports
            .checked_sub(general_data.pending_delegation_total)
            .error_log("Error: @calculating split lamports")?;
        general_data.is_t_stake_initialized = false;
        if split_lamports
            > LAMPORTS_PER_SOL
                + rent_data.minimum_balance(std::mem::size_of::<StakeState>() as usize)
        {
            log!(log_level, 3, "Splitting lamports ...");
            log!(log_level, 2, "Allocating account space ...");
            invoke_signed(
                &system_instruction::allocate(
                    t_withdraw_info.key,
                    std::mem::size_of::<StakeState>() as u64,
                ),
                &[t_withdraw_info.clone()],
                &[&[T_WITHDRAW_KEY.as_ref(), &[t_withdraw_bump]]],
            )
            .error_log("failed to allocate account space")?;
            log!(log_level, 2, "Account space allocated!!!");

            log!(log_level, 2, "Assigning account ...");
            invoke_signed(
                &system_instruction::assign(t_withdraw_info.key, &stake::program::id()),
                &[t_withdraw_info.clone()],
                &[&[T_WITHDRAW_KEY.as_ref(), &[t_withdraw_bump]]],
            )
            .error_log("failed to assign t_withdraw account")?;
            log!(log_level, 2, "Account assigned!!!");
            log!(
                log_level,
                1,
                "Split_Lamports: {:?}, stake_account_lamports: {:?}",
                split_lamports,
                stake_account_info.lamports()
            );
            log!(log_level, 2, "Splitting stake ...");
            invoke_signed(
                &split(
                    stake_account_info.key,
                    pd_pool_account_info.key,
                    split_lamports,
                    t_withdraw_info.key,
                ),
                &[
                    stake_account_info.clone(),
                    t_withdraw_info.clone(),
                    pd_pool_account_info.clone(),
                ],
                &[&[PD_POOL_ACCOUNT_KEY.as_ref(), &[pd_pool_bump]]],
            )
            .error_log("failed to split stake")?;
            log!(log_level, 2, "Stake split!!!");

            log!(log_level, 2, "Deactivating stake ...");
            invoke_signed(
                &solana_program::stake::instruction::deactivate_stake(
                    t_withdraw_info.key,
                    &pd_pool_pubkey,
                ),
                &[
                    t_withdraw_info.clone(),
                    sysvar_clock_info.clone(),
                    pd_pool_account_info.clone(),
                ],
                &[&[PD_POOL_ACCOUNT_KEY.as_ref(), &[pd_pool_bump]]],
            )
            .error_log("failed to deactivate stake from t_withdraw_info")?;
            log!(log_level, 2, "Stake deactivated!!!");

            general_data.pending_delegation_total = 0;
            general_data.dealloced = 0;
            general_data.rebalancing_data.pending_validator_rewards = val_owners_lamports;
            general_data.rebalancing_data.unclaimed_validator_rewards = 0;

            general_data.last_total_staked = stake_account_info
                .lamports()
                .checked_sub(split_lamports)
                .error_log("Error: @calculating last_total_staked")?;
        } else {
            log!(
                log_level,
                3,
                "Split lamports is less than min required to split ..."
            );
            if general_data.dealloced > general_data.pending_delegation_total {
                log!(log_level, 3, "Dealloced > Pending delegation Total ");
                general_data.dealloced = general_data
                    .dealloced
                    .checked_sub(general_data.pending_delegation_total)
                    .error_log("dealloced is less pending_delegation_total in general_data ")?;
                general_data.rebalancing_data.pending_validator_rewards = 0;

                general_data.rebalancing_data.unclaimed_validator_rewards = val_owners_lamports;
                general_data.pending_delegation_total = 0;
            } else {
                log!(log_level, 3, "Dealloced <= Pending delegation Total");
                general_data.pending_delegation_total = general_data
                    .pending_delegation_total
                    .checked_sub(general_data.dealloced)
                    .error_log("pending_delegation_total in general_data is less than that in general_data ")?;
                general_data.dealloced = 0;
                general_data.rebalancing_data.pending_validator_rewards = 0;
                general_data.rebalancing_data.unclaimed_validator_rewards = val_owners_lamports;
            }
        }
    }

    general_data.rebalancing_data.is_rebalancing_active = true;

    log!(log_level, 0, "begining serialization ...");
    general_data
        .serialize(&mut &mut general_account_info.data.borrow_mut()[..])
        .error_log("failed to serialize into general_account_info")?;
    log!(log_level, 4, "end of init_rebalance ...");
    Ok(())
}
