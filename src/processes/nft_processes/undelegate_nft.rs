use std::slice::Iter;

use crate::{
    error::InglError,
    log,
    processes::rewards_processes::nft_withdraw::nft_withdraw,
    state::{constants::*, FundsLocation, GeneralData, GovernanceData, NftData, ValidatorConfig},
    utils::{verify_nft_ownership, AccountInfoHelpers, OptionExt, ResultExt},
};

use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};
pub fn undelegate_nft(
    // TODO: prevent Undelegation while this NFT's vote is still countable in a non-finalized proposal. i.e. remove all votes on non-finalized proposals before undelegating.
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
    clock_is_from_account: bool,
    rent_is_from_account: bool,
) -> ProgramResult {
    log!(log_level, 4, "Undelegate nft...");
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let vote_account_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let nft_account_data_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;
    let system_program_account_info = next_account_info(account_info_iter)?;
    let authorized_withdrawer_info = next_account_info(account_info_iter)?;

    system_program_account_info
        .assert_key_match(&solana_program::system_program::id())
        .error_log("Error: @system_program_account_ingo key assertion")?;
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

    verify_nft_ownership(
        payer_account_info,
        mint_account_info,
        nft_account_data_info,
        associated_token_account_info,
        program_id,
    )?;

    let mut nft_withdraw_accounts = vec![
        payer_account_info.clone(),
        vote_account_info.clone(),
        general_account_info.clone(),
        config_account_info.clone(),
        authorized_withdrawer_info.clone(),
    ];
    if clock_is_from_account {
        nft_withdraw_accounts.push(next_account_info(account_info_iter)?.clone());
    }
    if rent_is_from_account {
        nft_withdraw_accounts.push(next_account_info(account_info_iter)?.clone());
    }
    nft_withdraw_accounts.extend(vec![
        associated_token_account_info.clone(),
        mint_account_info.clone(),
        nft_account_data_info.clone(),
        system_program_account_info.clone(),
    ]);
    let mut general_account_data = Box::new(GeneralData::parse(general_account_info, program_id)?);
    let mut nft_account_data = NftData::parse(&nft_account_data_info, program_id)
        .error_log("Error @gem_account_data_info decoding")?;

    let interested_proposals: Vec<&u32> = general_account_data
        .unfinalized_proposals
        .iter()
        .filter(|x| nft_account_data.all_votes.contains_key(*x))
        .collect();

    handle_vote_reversal(
        interested_proposals,
        account_info_iter,
        &mut nft_account_data,
        program_id,
        log_level,
    )?;

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

    //preventing attempt to withdraw when no rewards are available.
    if (general_account_data.vote_rewards.len() > 0)
        && (general_account_data
            .vote_rewards
            .last()
            .error_log("vote_rewards can't be empty here")?
            .epoch_number
            > interested_epoch)
    {
        nft_withdraw(
            program_id,
            &nft_withdraw_accounts,
            1,
            log_level,
            clock_is_from_account,
            rent_is_from_account,
        )
        .error_log("Error: @nft_withdraw")?;
    }
    let config_data = Box::new(ValidatorConfig::parse(config_account_info, program_id)?);

    general_account_data.total_delegated = general_account_data
        .total_delegated
        .checked_sub(config_data.unit_backing)
        .error_log("Error: @ general_data.total_delegated recalc")?;

    if general_account_data.pending_delegation_total >= config_data.unit_backing {
        general_account_data.pending_delegation_total = general_account_data
            .pending_delegation_total
            .checked_sub(config_data.unit_backing)
            .error_log("Error: @ general_data.pending_delegation_total recalc")?;
    } else {
        general_account_data.dealloced = general_account_data
            .dealloced
            .checked_add(config_data.unit_backing)
            .error_log("Error: @ general_data.dealloced_total recalc")?;
    }

    match nft_account_data.funds_location {
        FundsLocation::Delegated => {
            nft_account_data.funds_location = FundsLocation::Undelegated;
        }
        _ => Err(InglError::InvalidFundsLocation.utilize("gem's funds location."))?,
    }

    general_account_data
        .serialize(&mut &mut general_account_info.data.borrow_mut()[..])
        .error_log("Error: @general_data serialization")?;
    nft_account_data
        .serialize(&mut &mut nft_account_data_info.data.borrow_mut()[..])
        .error_log("Error: @gem_account_data serialization")?;

    Ok(())
}

fn handle_vote_reversal(
    interested_proposals: Vec<&u32>,
    account_info_iter: &mut Iter<AccountInfo>,
    nft_data: &mut NftData,
    program_id: &Pubkey,
    log_level: u8,
) -> ProgramResult {
    for proposal_numeration in interested_proposals.iter() {
        log!(log_level, 2, "proposal_numeration: {}", proposal_numeration);
        let proposal_account_info = next_account_info(account_info_iter)?;
        proposal_account_info
            .assert_owner(program_id)
            .error_log("Error: Proposal account is not owned by the program")?;
        let (_proposal_id, _proposal_bump) = proposal_account_info
            .assert_seed(
                program_id,
                &[
                    INGL_PROPOSAL_KEY.as_ref(),
                    &proposal_numeration.to_be_bytes(),
                ],
            )
            .error_log("failed to assert pda input for proposal_account_info")?;

        let mut governance_data =
            Box::new(GovernanceData::parse(proposal_account_info, program_id)?);

        match governance_data.votes.remove(&nft_data.numeration) {
            None => {
                Err(InglError::InvalidData.utilize("vote to remove not found in governance data"))?
            }
            Some(vote) => match nft_data.all_votes.remove(proposal_numeration) {
                None => {
                    Err(InglError::InvalidData.utilize("vote to remove not found in nft data"))?
                }
                Some(nft_data_vote) => assert_eq!(nft_data_vote, vote),
            },
        }
        governance_data
            .serialize(&mut &mut proposal_account_info.data.borrow_mut()[..])
            .error_log("Error: @governance_data serialization")?;
    }

    Ok(())
}
