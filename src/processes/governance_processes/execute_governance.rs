use std::slice::Iter;

use crate::{
    error::InglError,
    instruction::{vote_authorize, vote_update_commission, vote_update_validator_identity},
    log,
    state::{
        constants::*, ConfigAccountType, GeneralData, GovernanceData, GovernanceType,
        ValidatorConfig, VoteAccountGovernance, VoteAuthorize,
    },
    utils::{AccountInfoHelpers, OptionExt, ResultExt},
};

use borsh::BorshSerialize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    bpf_loader_upgradeable,
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke_signed,
    pubkey::Pubkey,
    sysvar::{self, Sysvar},
};

pub fn execute_governance(
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

    let clock_data =
        Clock::from_account_info(sysvar_clock_info).error_log("failed to get clock data")?;

    let mut governance_data = GovernanceData::decode(proposal_account_info)?;
    let mut config_data = Box::new(ValidatorConfig::decode(ingl_config_account)?);
    let general_data = Box::new(GeneralData::decode(general_account_info)?);

    if governance_data.is_still_ongoing == true {
        Err(InglError::TooEarly.utilize("This proposal is currently still ongoing."))?
    }
    if governance_data.is_proposal_executed == true {
        Err(InglError::TooLate.utilize("This proposal has already been executed."))?
    }
    if (clock_data.unix_timestamp as u32)
        < (governance_data
            .clone()
            .date_finalized
            .error_log("Proposal must be finalized")?
            + 86400 * 30)
    {}

    match governance_data.did_proposal_pass {
        Some(x) => {
            if x == false {
                Err(InglError::InvalidData.utilize("This proposal was not passed."))?
            }
        }
        None => Err(InglError::TooEarly.utilize("This proposal is currently still ongoing."))?,
    }

    match governance_data.clone().governance_type {
        GovernanceType::ConfigAccount(config_governance_type) => {
            match config_governance_type {
                ConfigAccountType::MaxPrimaryStake(x) => {
                    config_data.max_primary_stake = x;
                }
                ConfigAccountType::NftHolderShare(x) => {
                    config_data.nft_holders_share = x;
                }
                ConfigAccountType::InitialRedemptionFee(x) => {
                    config_data.initial_redemption_fee = x;
                }
                ConfigAccountType::RedemptionFeeDuration(x) => {
                    config_data.redemption_fee_duration = x;
                }
                ConfigAccountType::ValidatorName(x) => {
                    config_data.validator_name = x;
                }
                ConfigAccountType::TwitterHandle(x) => {
                    config_data.twitter_handle = x;
                }
                ConfigAccountType::DiscordInvite(x) => {
                    config_data.discord_invite = x;
                }
            }
            config_data
                .validate_data()
                .error_log("Invalid data in config account")?;
        }
        GovernanceType::ProgramUpgrade {
            buffer_account,
            code_link: _,
        } => handle_program_upgrade(program_id, account_info_iter, buffer_account, log_level)?,
        GovernanceType::VoteAccountGovernance(vote_account_governance_type) => {
            handle_vote_account_governance_change(
                program_id,
                account_info_iter,
                vote_account_governance_type.clone(),
                log_level,
            )?;
            match vote_account_governance_type {
                VoteAccountGovernance::Commission(x) => {
                    config_data.commission = x;
                }
                VoteAccountGovernance::ValidatorID(x) => {
                    config_data.validator_id = x;
                    if general_data.last_validated_validator_id_proposal > proposal_numeration {
                        Err(InglError::TooLate.utilize("The Time to Execute Proposal has passed."))?
                    }
                }
            }
        }
    }

    governance_data.is_proposal_executed = true;
    log!(log_level, 0, "serialization only left");
    governance_data
        .serialize(&mut &mut proposal_account_info.data.borrow_mut()[..])
        .error_log("failed to serialize into proposal_account_info")?;
    config_data
        .serialize(&mut &mut ingl_config_account.data.borrow_mut()[..])
        .error_log("failed to serialize into ingl_config_account")?;
    log!(log_level, 4, "Done with upgrade proposal finalization !!!");
    Ok(())
}

pub fn handle_program_upgrade(
    program_id: &Pubkey,
    account_info_iter: &mut Iter<AccountInfo>,
    buffer_account: Pubkey,
    log_level: u8,
) -> ProgramResult {
    log!(log_level, 3, "initiating program upgrade  ");
    let upgraded_program_info = next_account_info(account_info_iter)?;
    let buffer_address_info = next_account_info(account_info_iter)?;
    let spilling_address_info = next_account_info(account_info_iter)?;
    let programdata_info = next_account_info(account_info_iter)?;
    let authority_address_info = next_account_info(account_info_iter)?;
    let sysvar_rent_info = next_account_info(account_info_iter)?;
    let sysvar_clock_info = next_account_info(account_info_iter)?;

    let (expected_authority_address, epa_bump) = authority_address_info
        .assert_seed(program_id, &[INGL_PROGRAM_AUTHORITY_KEY.as_ref()])
        .error_log("failed to assert_pda_input for authority_address_info")?;

    buffer_address_info
        .assert_key_match(&buffer_account)
        .error_log("Error @ Buffer address verification")?;
    sysvar_clock_info
        .assert_key_match(&sysvar::clock::id())
        .error_log("Error @ Clock address verification")?;
    sysvar_rent_info
        .assert_key_match(&sysvar::rent::id())
        .error_log("Error @ Rent address verification")?;
    upgraded_program_info
        .assert_key_match(&program_id)
        .error_log("Error @ Program address verification")?;
    programdata_info
        .assert_seed(&bpf_loader_upgradeable::id(), &[program_id.as_ref()])
        .error_log("Error @ Program data address verification")?;
    programdata_info
        .assert_owner(&bpf_loader_upgradeable::id())
        .error_log("Error @ Program data owner verification")?;
    buffer_address_info
        .assert_owner(&bpf_loader_upgradeable::id())
        .error_log("Error @ Buffer address owner verification")?;
    sysvar_clock_info
        .assert_owner(&sysvar::id())
        .error_log("Error @ Clock address owner verification")?;
    sysvar_rent_info
        .assert_owner(&sysvar::id())
        .error_log("Error @ Rent address owner verification")?;

    log!(log_level, 2, "Proposal was passed. Upgrading Program ...");
    invoke_signed(
        &bpf_loader_upgradeable::upgrade(
            upgraded_program_info.key,
            &buffer_address_info.key,
            &expected_authority_address,
            &spilling_address_info.key,
        ),
        &[
            programdata_info.clone(),
            upgraded_program_info.clone(),
            buffer_address_info.clone(),
            spilling_address_info.clone(),
            sysvar_rent_info.clone(),
            sysvar_clock_info.clone(),
            authority_address_info.clone(),
        ],
        &[&[
            INGL_PROGRAM_AUTHORITY_KEY.as_ref(),
            &upgraded_program_info.key.to_bytes(),
            &[epa_bump],
        ]],
    )
    .error_log("failed to upgrade program")?;
    log!(log_level, 3, "Program Upgraded !!!");

    Ok(())
}

pub fn handle_vote_account_governance_change(
    program_id: &Pubkey,
    account_info_iter: &mut Iter<AccountInfo>,
    governance_type: VoteAccountGovernance,
    log_level: u8,
) -> ProgramResult {
    let authorized_withdrawer_info = next_account_info(account_info_iter)?;
    let vote_account_info = next_account_info(account_info_iter)?;

    vote_account_info
        .assert_seed(program_id, &[VOTE_ACCOUNT_KEY.as_ref()])
        .error_log("failed to assert_pda_input for vote_account_info")?;
    vote_account_info
        .assert_owner(&solana_program::vote::program::id())
        .error_log("failed to assert_owner for vote_account_info")?;
    let (_, aw_bump) = authorized_withdrawer_info
        .assert_seed(program_id, &[AUTHORIZED_WITHDRAWER_KEY.as_ref()])
        .error_log("failed to assert_pda_input for authorized_withdrawer_info")?;

    match governance_type {
        VoteAccountGovernance::ValidatorID(new_validator_id) => {
            let sysvar_clock_info = next_account_info(account_info_iter)?;
            let new_validator_id_info = next_account_info(account_info_iter)?;
            new_validator_id_info
                .assert_key_match(&new_validator_id)
                .error_log("Error @ New Validator ID address verification")?;
            sysvar_clock_info
                .assert_key_match(&sysvar::clock::id())
                .error_log("Error @ Clock address verification")?;
            sysvar_clock_info
                .assert_owner(&sysvar::id())
                .error_log("Error @ Clock address owner verification")?;
            log!(
                log_level,
                3,
                "Initiating authorized_voter change invocation ..."
            );
            invoke_signed(
                &vote_authorize(
                    vote_account_info.key,
                    authorized_withdrawer_info.key,
                    &new_validator_id,
                    VoteAuthorize::Voter,
                ),
                &[
                    vote_account_info.clone(),
                    sysvar_clock_info.clone(),
                    authorized_withdrawer_info.clone(),
                ],
                &[&[AUTHORIZED_WITHDRAWER_KEY.as_ref(), &[aw_bump]]],
            )?;
            log!(log_level, 3, "Changed authorized_voter !!!");
            invoke_signed(
                &vote_update_validator_identity(
                    vote_account_info.key,
                    authorized_withdrawer_info.key,
                    new_validator_id_info.key,
                ),
                &[
                    vote_account_info.clone(),
                    new_validator_id_info.clone(),
                    authorized_withdrawer_info.clone(),
                ],
                &[&[AUTHORIZED_WITHDRAWER_KEY.as_ref(), &[aw_bump]]],
            )?;
            log!(log_level, 3, "Changed Validator ID !!!");
        }
        VoteAccountGovernance::Commission(new_commission) => {
            log!(log_level, 3, "Initiating commission change invocation ...");
            invoke_signed(
                &vote_update_commission(
                    vote_account_info.key,
                    authorized_withdrawer_info.key,
                    new_commission,
                ),
                &[
                    vote_account_info.clone(),
                    authorized_withdrawer_info.clone(),
                ],
                &[&[AUTHORIZED_WITHDRAWER_KEY.as_ref(), &[aw_bump]]],
            )?;
            log!(log_level, 3, "Changed Commission !!!");
        }
    }
    Ok(())
}
