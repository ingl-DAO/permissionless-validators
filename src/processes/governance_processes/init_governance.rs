use std::collections::BTreeMap;

use crate::{
    error::InglError,
    log,
    state::{
        constants::*, GeneralData, GovernanceData, GovernanceType, UpgradeableLoaderState,
        ValidatorConfig, VoteAccountGovernance,
    },
    utils::{
        get_clock_data, get_rent_data, verify_nft_ownership, AccountInfoHelpers, OptionExt,
        PubkeyHelpers, ResultExt,
    },
};

use borsh::BorshSerialize;

use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    bpf_loader_upgradeable,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction,
};

pub fn create_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    governance_type: GovernanceType,
    log_level: u8,
    clock_is_from_account: bool,
    rent_is_from_account: bool,
) -> ProgramResult {
    log!(log_level, 4, "Starting create_governance_proposal ... ");
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let vote_account_info = next_account_info(account_info_iter)?;
    let proposal_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let nft_account_data_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;

    verify_nft_ownership(
        payer_account_info,
        mint_account_info,
        nft_account_data_info,
        associated_token_account_info,
        program_id,
    )?;

    vote_account_info
        .assert_seed(program_id, &[VOTE_ACCOUNT_KEY.as_ref()])
        .error_log("failed at vote account seed assertion")?;
    general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])
        .error_log("failed at general account seed assertion")?;
    general_account_info
        .assert_owner(program_id)
        .error_log("failed at general account owner assertion")?;
    config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("failed at config account seed assertion")?;
    config_account_info
        .assert_owner(program_id)
        .error_log("failed at config account owner assertion")?;

    let config_data = Box::new(ValidatorConfig::parse(config_account_info, program_id)?);

    let clock_data = get_clock_data(account_info_iter, clock_is_from_account)?;

    let rent_data = get_rent_data(account_info_iter, rent_is_from_account)?;

    log!(log_level, 0, "Done with account collection");

    let buffer_address_info;

    match governance_type.clone() {
        GovernanceType::ProgramUpgrade {
            buffer_account,
            code_link: _,
        } => {
            buffer_address_info = next_account_info(account_info_iter)?;
            buffer_address_info
                .assert_key_match(&buffer_account)
                .error_log("Error @ Buffer account must match the account info")?;
            buffer_address_info
                .assert_owner(&bpf_loader_upgradeable::id())
                .error_log("buffer_address_info is not owned by bpf_loader_upgradeable")?;
            let buffer_data: UpgradeableLoaderState =
                bincode::deserialize(&buffer_address_info.data.borrow())
                    .expect("failed to deserialize buffer_address_info data");
            match buffer_data {
                UpgradeableLoaderState::Buffer { authority_address } => {
                    let (expected_authority_address, _epda_bump) = Pubkey::find_program_address(
                        &[INGL_PROGRAM_AUTHORITY_KEY.as_ref()],
                        program_id,
                    );
                    authority_address
                        .error_log("Program must have an authority address")?
                        .assert_match(&expected_authority_address)
                        .error_log("Error @ Authority must the correct program's PDA")?;
                }
                _ => return Err(InglError::ExpectedBufferAccount.utilize("")),
            }
        }

        GovernanceType::VoteAccountGovernance(x) => match x {
            VoteAccountGovernance::ValidatorID(_) => {
                if config_data.is_validator_id_switchable == false {
                    return Err(InglError::InvalidData
                        .utilize("Validator Id for this Validator Instance is not switchable"));
                }
            }
            _ => (),
        },
        _ => (),
    }

    let mut general_account_data = Box::new(GeneralData::parse(general_account_info, program_id)?);
    let (_proposal_id, proposal_bump) = proposal_account_info
        .assert_seed(
            program_id,
            &[
                INGL_PROPOSAL_KEY.as_ref(),
                &general_account_data.proposal_numeration.to_be_bytes(),
            ],
        )
        .error_log("failed to assert pda input for proposal_account_info")?;

    log!(log_level, 0, "Done with account assertions");

    let governance_data = GovernanceData {
        validation_phrase: GOVERNANCE_DATA_VAL_PHRASE,
        expiration_time: clock_data.unix_timestamp as u32 + 60 * 60 * 24 * 30,
        is_still_ongoing: true,
        date_finalized: None,
        did_proposal_pass: None,
        is_proposal_executed: false,
        votes: BTreeMap::new(),
        governance_type: governance_type,
    };
    governance_data
        .verify()
        .error_log("governance_data is invalid")?;

    let space = governance_data.get_space();
    let lamports = rent_data.minimum_balance(space);

    log!(log_level, 2, "Creating proposal account ...");
    invoke_signed(
        &system_instruction::create_account(
            payer_account_info.key,
            proposal_account_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[payer_account_info.clone(), proposal_account_info.clone()],
        &[&[
            INGL_PROGRAM_AUTHORITY_KEY.as_ref(),
            &general_account_data.proposal_numeration.to_be_bytes(),
            &[proposal_bump],
        ]],
    )
    .error_log("failed to create proposal account")?;
    log!(log_level, 2, "Created proposal account !!!");

    log!(log_level, 2, "Transfering Spam prevention Sol ...");
    invoke(
        &system_instruction::transfer(
            payer_account_info.key,
            vote_account_info.key,
            LAMPORTS_PER_SOL.checked_mul(2).unwrap(),
        ),
        &[payer_account_info.clone(), vote_account_info.clone()],
    )
    .error_log("failed to transfer spam prevention sol")?;
    log!(log_level, 2, "Transferred Spam prevention Sol !!!");

    general_account_data.proposal_numeration += 1;
    log!(log_level, 0, "Serializing data ...");
    governance_data
        .serialize(&mut &mut proposal_account_info.data.borrow_mut()[..])
        .error_log("failed to serialize into proposal_account_info")?;
    general_account_data
        .serialize(&mut &mut general_account_info.data.borrow_mut()[..])
        .error_log("failed to serialize into global_gem_account_info")?;
    log!(log_level, 4, "Done with create_governance_proposal !!!");
    Ok(())
}
