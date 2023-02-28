use crate::{
    error::InglError,
    log,
    state::{constants::*, GeneralData, ValidatorConfig, VoteReward},
    utils::{get_clock_data, get_rent_data, AccountInfoHelpers, OptionExt, ResultExt},
};

use borsh::BorshSerialize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction, vote::instruction::withdraw,
};

pub fn process_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
    clock_is_from_account: bool,
    rent_is_from_account: bool,
) -> ProgramResult {
    log!(log_level, 4, "initiating process_rewards ...");
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let validator_info = next_account_info(account_info_iter)?;
    let vote_account_info = next_account_info(account_info_iter)?;
    let authorized_withdrawer_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;
    let ingl_team_account_info = next_account_info(account_info_iter)?;

    let clock_data = get_clock_data(account_info_iter, clock_is_from_account)?;

    let rent_data = get_rent_data(account_info_iter, rent_is_from_account)?;

    log!(log_level, 0, "Done with main accounts collection");

    vote_account_info
        .assert_owner(&solana_program::vote::program::id())
        .error_log("Error @ vote_account ownership assertion")?;

    config_account_info
        .assert_owner(program_id)
        .error_log("Error @ config_account ownership assertion")?;
    general_account_info
        .assert_owner(program_id)
        .error_log("Error @ general_account ownership assertion")?;

    config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("Error @ config_account seed assertion")?;
    general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])
        .error_log("Error @ general_account seed assertion")?;

    ingl_team_account_info
        .assert_key_match(&team::id())
        .error_log("Error @ ingl_team_account key match assertion")?;

    let config_data = Box::new(ValidatorConfig::parse(config_account_info, program_id)?);
    let mut general_data = Box::new(GeneralData::parse(general_account_info, program_id)?);
    vote_account_info
        .assert_key_match(&config_data.vote_account)
        .error_log("Error @ Vote account address verification")?;

    let validator_id = config_data.validator_id; // TODO: Stop using config account for validator_id storage, and fetch it directly from the vote account data
    validator_info
        .assert_key_match(&validator_id)
        .error_log("failed to assert pubkeys exactitude for validator_info")?;

    let (authorized_withdrawer, authorized_withdrawer_bump) = authorized_withdrawer_info
        .assert_seed(program_id, &[AUTHORIZED_WITHDRAWER_KEY.as_ref()])
        .error_log("failed to assert pda input for authorized_withdrawer_info")?;
    let reward_lamports = vote_account_info
        .lamports()
        .checked_sub(rent_data.minimum_balance(vote_account_info.data_len()))
        .error_log("There is not enough lamports in the vote account to withdraw the funds")?;
    let one_percent: u64 = reward_lamports
        .checked_div(100)
        .error_log("Failed during one percent calculation")?;

    log!(
        log_level,
        2,
        "Withdrawing the funds from the vote account ..."
    );
    invoke_signed(
        &withdraw(
            vote_account_info.key,
            authorized_withdrawer_info.key,
            reward_lamports,
            &authorized_withdrawer,
        ),
        &[
            vote_account_info.clone(),
            authorized_withdrawer_info.clone(),
            authorized_withdrawer_info.clone(),
        ],
        &[&[
            AUTHORIZED_WITHDRAWER_KEY.as_ref(),
            &[authorized_withdrawer_bump],
        ]],
    )
    .error_log("failed to invoke vote_withdraw")?;
    log!(log_level, 2, "Funds withdrawn from the vote account!!!");

    match general_data.vote_rewards.last() {
        Some(vote_reward) => {
            if vote_reward.epoch_number >= clock_data.epoch {
                Err(InglError::TooEarly.utilize("processing reward"))?
            }
        }
        None => {}
    }
    let team_share = one_percent.checked_mul(team::TEAM_SHARE).unwrap();

    log!(
        log_level,
        2,
        "Transferring the funds to the ingl team account ..."
    );
    invoke_signed(
        &system_instruction::transfer(
            authorized_withdrawer_info.key,
            ingl_team_account_info.key,
            team_share,
        ),
        &[
            authorized_withdrawer_info.clone(),
            ingl_team_account_info.clone(),
        ],
        &[&[
            AUTHORIZED_WITHDRAWER_KEY.as_ref(),
            &[authorized_withdrawer_bump],
        ]],
    )
    .error_log(
        "failed to transfer funds from authorized_withdrawer_info to ingl_team_account_info",
    )?;
    log!(
        log_level,
        2,
        "Funds transferred to the ingl team account!!!"
    );

    log!(
        log_level,
        2,
        "Transferring the funds to the validator's account ..."
    );

    let remaining_reward = reward_lamports
        .checked_sub(team_share)
        .error_log("Error calculating remaining rewards")?;
    let r_one_percent = remaining_reward
        .checked_div(100)
        .error_log("Error calculating r_one_percent")?;
    let validator_share = r_one_percent
        .checked_mul((100 - config_data.nft_holders_share).into())
        .error_log("Error calculating validator_share")?;
    let nft_holders_share = remaining_reward
        .checked_sub(validator_share)
        .error_log("Error calculating nft_holders_share")?;
    invoke_signed(
        &system_instruction::transfer(
            authorized_withdrawer_info.key,
            validator_info.key,
            validator_share,
        ),
        &[authorized_withdrawer_info.clone(), validator_info.clone()],
        &[&[
            AUTHORIZED_WITHDRAWER_KEY.as_ref(),
            &[authorized_withdrawer_bump],
        ]],
    )
    .error_log("failed to transfer funds from authorized_withdrawer_info to validator_info")?;
    log!(
        log_level,
        2,
        "Funds transferred to the validator's account!!!"
    );

    let new_space = general_account_info.data.borrow().len() + 32;
    let lamports = rent_data
        .minimum_balance(new_space)
        .checked_sub(rent_data.minimum_balance(general_account_info.data.borrow().len()))
        .unwrap();

    invoke(
        &system_instruction::transfer(payer_account_info.key, general_account_info.key, lamports),
        &[payer_account_info.clone(), general_account_info.clone()],
    )
    .error_log(
        "failed to transfer for reallaocating ingl vote data account size @system_program invoke",
    )?;

    general_account_info
        .realloc(new_space, false)
        .error_log("failed to realloc ingl vote data account data account size")?;

    general_data.vote_rewards.push(VoteReward {
        epoch_number: clock_data.epoch,
        total_stake: general_data.total_delegated,
        total_reward: reward_lamports,
        nft_holders_reward: nft_holders_share,
    });
    general_data.last_withdraw_epoch = clock_data.epoch;

    general_data
        .serialize(&mut &mut general_account_info.data.borrow_mut()[..])
        .error_log("failed to serialize ingl_vote_data_account_info")?;

    log!(log_level, 4, "Processing reward finished!!!");
    Ok(())
}
