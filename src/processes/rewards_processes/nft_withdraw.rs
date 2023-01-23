use crate::{
    error::InglError,
    log,
    state::{constants::*, FundsLocation, GeneralData, NftData, ValidatorConfig},
    utils::{
        get_clock_data, get_rent_data, verify_nft_ownership, AccountInfoHelpers, OptionExt,
        ResultExt,
    },
};

use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction, vote,
};

///Withdraws the rewards accrued from the epoch after the gem was delegated to the last epoch the process_rewards instruction was run
pub fn nft_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    cnt: u8,
    log_level: u8,
    clock_is_from_account: bool,
    rent_is_from_account: bool,
) -> ProgramResult {
    log!(
        log_level,
        0,
        "initializng NFT withdraw instruction ... cnt: {}",
        cnt
    );
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let vote_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;
    let authorized_withdrawer_info = next_account_info(account_info_iter)?;

    let clock_data = get_clock_data(account_info_iter, clock_is_from_account)?;
    let rent_data = get_rent_data(account_info_iter, rent_is_from_account)?;

    log!(log_level, 0, "Done with main account collection");

    let (_general_account_pubkey, _general_account_bump) = general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])
        .error_log("Error: failed to assert pda input for general_account_info")?;
    vote_account_info
        .assert_seed(program_id, &[VOTE_ACCOUNT_KEY.as_ref()])
        .error_log("Error: vote_account_info must be the expected pda")?;
    vote_account_info
        .assert_owner(&vote::program::id())
        .error_log("Error: vote_account_info must be owned by the vote_program")?;
    general_account_info
        .assert_owner(program_id)
        .error_log("Error: general_account_info must be owned by the program")?;
    config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("Error @ Config account pda assertion")?;

    let general_data = Box::new(GeneralData::parse(general_account_info, program_id)?);
    let config_data = Box::new(ValidatorConfig::parse(config_account_info, program_id)?);

    let (_authorized_withdrawer, authorized_withdrawer_bump) = authorized_withdrawer_info
        .assert_seed(program_id, &[AUTHORIZED_WITHDRAWER_KEY.as_ref()])
        .error_log("Error: failed to assert pda input for authorized_withdrawer_info")?;

    payer_account_info
        .assert_signer()
        .error_log("Error: Payer must be Signer, couldn't find its signature")?;

    log!(log_level, 0, "Done with main account assertions");
    let mut general_rewards: u64 = 0;
    for num in 0..cnt {
        log!(log_level, 1, "gem_numeration {}", num);
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let mint_account_info = next_account_info(account_info_iter)?;
        let nft_account_data_info = next_account_info(account_info_iter)?;

        verify_nft_ownership(
            payer_account_info,
            mint_account_info,
            nft_account_data_info,
            associated_token_account_info,
            program_id,
        )
        .error_log("Error @ nft ownership verification")?;

        let mut ingl_nft_data = NftData::parse(nft_account_data_info, program_id)
            .error_log("Error: @nft_account_data_info deserialization")?;

        if let FundsLocation::Delegated = ingl_nft_data.funds_location {
        } else {
            Err(InglError::InvalidFundsLocation.utilize("Gem's fund location"))?
        }

        let new_space = nft_account_data_info.data.borrow().len() + 8;
        let lamports = rent_data
            .minimum_balance(new_space)
            .checked_sub(rent_data.minimum_balance(nft_account_data_info.data.borrow().len()))
            .unwrap();

        invoke(
            &system_instruction::transfer(
                payer_account_info.key,
                nft_account_data_info.key,
                lamports,
            ),
            &[payer_account_info.clone(), nft_account_data_info.clone()],
        )
        .error_log(
            "failed to transfer for reallaocating_gem_data_account_size @system_program invoke",
        )?;
        nft_account_data_info
            .realloc(new_space, false)
            .error_log("Error: @realloc of nft_account_data_info")?;
        let total_reward =
            calculate_total_reward(&ingl_nft_data, &general_data, &config_data, log_level)
                .error_log("Error: @calculate_total_reward")?;
        ingl_nft_data.last_withdrawal_epoch = Some(clock_data.epoch);
        ingl_nft_data.all_withdraws.push(total_reward as u64);
        general_rewards = general_rewards.checked_add(total_reward as u64).unwrap();
        ingl_nft_data
            .serialize(&mut &mut nft_account_data_info.data.borrow_mut()[..])
            .error_log("Error: @nft_account_data_info serialization")?;
    }
    log!(log_level, 2, "Transfering Gem's Reward ...");
    invoke_signed(
        &system_instruction::transfer(
            authorized_withdrawer_info.key,
            payer_account_info.key,
            general_rewards,
        ),
        &[
            authorized_withdrawer_info.clone(),
            payer_account_info.clone(),
        ],
        &[&[
            AUTHORIZED_WITHDRAWER_KEY.as_ref(),
            &[authorized_withdrawer_bump],
        ]],
    )
    .error_log("Error: transfer from authorized_withdrawer to payer")?;
    log!(log_level, 2, "Gem's Reward Transfered!!!");

    Ok(())
}

///UNCHECKED. Calculates the total reward for a specific gem, for the epochs that the gem was delegated without rewards being withdrawn.
pub fn calculate_total_reward(
    nft_account_data: &NftData,
    general_data: &GeneralData,
    config_data: &ValidatorConfig,
    log_level: u8,
) -> Result<u128, ProgramError> {
    let interested_epoch = if let Some(tmp) = nft_account_data.last_withdrawal_epoch {
        tmp.max(
            nft_account_data
                .last_delegation_epoch
                .error_log("Error: Last delegation epoch can't be None at this stage")?,
        )
    } else {
        nft_account_data
            .last_delegation_epoch
            .error_log("Error: Last delegation epoch can't be None at this stage")?
    };
    //TODO: users could delegate right before the end of an epoch, then after the epoch ends, they could run process_rewards, then run the withdraw function. This could be a problem on validators that have no early redemption fees. This could be an abuse of the system. A solution might be to make sure than x.epoch_number > 1 + interested_epoch, find x in the line below.
    let interested_index = general_data.vote_rewards.iter().position(|x| x.epoch_number > interested_epoch).error_log("Error: couldn't find an epoch greater than both the last delegation epoch and the last withdrrawal epoch.")?;
    log!(
        log_level,
        1,
        "interested_index: {:?}, interested_epoch: {:?}",
        interested_index,
        interested_epoch
    );
    let mut total_reward: u128 = 0;
    for i in interested_index..general_data.vote_rewards.len() {
        let epoch_reward = general_data.vote_rewards[i];
        log!(log_level, 1, "epoch_reward: {:?}", epoch_reward);
        total_reward = total_reward
            .checked_add(
                (epoch_reward
                    .nft_holders_reward as u128).checked_mul(config_data.unit_backing as u128)
                    .error_log("Error @ unit backing multiplication")?
                    .checked_div(epoch_reward.total_stake as u128)
                    .error_log("Error calculating unit reward for an epoch")?
                    as u128
            )
            .error_log("Error: total_reward")?;
    }

    Ok(total_reward)
}
