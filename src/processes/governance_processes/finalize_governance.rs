use crate::{
    error::InglError,
    log,
    state::{
        constants::*, GeneralData, GovernanceData, GovernanceType, ValidatorConfig,
        VoteAccountGovernance,
    },
    utils::{AccountInfoHelpers, OptionExt, ResultExt},
};

use borsh::BorshSerialize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    sysvar::{self, Sysvar},
};

//TODO: Make authorized voter change proposal instantly executable after finalization.
pub fn finalize_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    proposal_numeration: u32,
    log_level: u8,
) -> ProgramResult {
    log!(
        log_level,
        4,
        "Initiating Finalize program upgrade proposal ..."
    );
    let account_info_iter = &mut accounts.iter();
    let _payer_account_info = next_account_info(account_info_iter)?;
    let sysvar_rent_info = next_account_info(account_info_iter)?;
    let sysvar_clock_info = next_account_info(account_info_iter)?;
    let proposal_account_info = next_account_info(account_info_iter)?;
    let ingl_config_account = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;

    log!(log_level, 0, "Done with account collection");

    let (_proposal_id, _proposal_bump) = proposal_account_info
        .assert_seed(
            program_id,
            &[
                INGL_PROPOSAL_KEY.as_ref(),
                &proposal_numeration.to_be_bytes(),
            ],
        )
        .error_log("failed to assert_pda_input for proposal_account_info")?;

    let (_general_account_data, _general_account_data_bump) = general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])
        .error_log("failed to assert_pda_input for general_account_info")?;

    let (_ingl_config_id, _ingl_config_bump) = ingl_config_account
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("failed to assert_pda_input for ingl_config_account")?;

    sysvar_clock_info
        .assert_key_match(&sysvar::clock::id())
        .error_log("sent clock address is dissimilar from the expected one")?;
    sysvar_rent_info
        .assert_key_match(&sysvar::rent::id())
        .error_log("sent rent address is dissimilar from the expected one")?;

    let clock_data =
        Clock::from_account_info(sysvar_clock_info).error_log("failed to get clock data")?;

    let mut governance_data = Box::new(GovernanceData::parse(proposal_account_info, program_id)?);
    let config_data = Box::new(ValidatorConfig::parse(ingl_config_account, program_id)?);
    let mut general_data = Box::new(GeneralData::parse(general_account_info, program_id)?);
    if !general_data
        .unfinalized_proposals
        .remove(&proposal_numeration)
    {
        Err(InglError::InvalidData
            .utilize("Could not find the proposal in the unfinalized_proposals"))?
    }

    if governance_data.is_still_ongoing == false {
        Err(InglError::TooLate.utilize("This proposal is currently Closed"))?
    }
    log!(log_level, 0, "Done with account validations ...");
    let total_votes_expected = config_data
        .max_primary_stake
        .checked_div(config_data.unit_backing)
        .error_log("failed to calculate total_votes_expected")?
        as u32;
    let mut total_yes_votes: u32 = 0;
    let mut total_no_votes: u32 = 0;
    for vote in governance_data.votes.values() {
        if *vote {
            total_yes_votes += 1;
        } else {
            total_no_votes += 1;
        }
    }

    if total_no_votes + total_yes_votes
        <= (config_data.proposal_quorum as u32)
            .checked_mul(total_votes_expected)
            .error_log("Error at quorum mult")?
            .checked_div(100)
            .error_log("Error Calculating quorum percentage")?
    {
        Err(InglError::NotEnoughVotes.utilize(""))?
    }

    if (total_no_votes * 100)
        .checked_div(total_votes_expected)
        .error_log("Error Calculating Dissaproval Percentage")?
        > 20
        || (governance_data.expiration_time < clock_data.unix_timestamp as u32)
    {
        governance_data.did_proposal_pass = Some(false);
    } else if total_yes_votes as f64 >= total_votes_expected as f64 * GOVERNANCE_EXECUTION_THRESHOLD
    {
        governance_data.did_proposal_pass = Some(true);
        match governance_data.clone().governance_type {
            GovernanceType::ProgramUpgrade {
                buffer_account: _,
                code_link: _,
            } => {
                general_data.last_feeless_redemption_date =
                    clock_data.unix_timestamp as u32 + FEELESS_REDEMPTION_PERIOD;
            }
            GovernanceType::VoteAccountGovernance(x) => match x {
                VoteAccountGovernance::ValidatorID(_) => {
                    general_data.last_validated_validator_id_proposal = proposal_numeration
                }
                _ => (),
            },
            _ => (),
        }
    }
    governance_data.is_still_ongoing = false;
    log!(log_level, 0, "serialization only left");
    governance_data
        .serialize(&mut &mut proposal_account_info.data.borrow_mut()[..])
        .error_log("failed to serialize into proposal_account_info")?;
    general_data
        .serialize(&mut &mut general_account_info.data.borrow_mut()[..])
        .error_log("failed to serialize into general_account_info")?;
    log!(log_level, 4, "Done with upgrade proposal finalization !!!");
    Ok(())
}
