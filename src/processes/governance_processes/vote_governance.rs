use crate::{
    error::InglError,
    log,
    state::{constants::*, GovernanceData, NftData},
    utils::{get_clock_data, get_rent_data, verify_nft_ownership, AccountInfoHelpers, ResultExt},
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    pubkey::Pubkey,
    system_instruction,
};

pub fn vote_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    numeration: u32,
    vote: bool,
    cnt: u8,
    log_level: u8,
    clock_is_from_account: bool,
    rent_is_from_account: bool,
) -> ProgramResult {
    log!(log_level, 4, "Initiating Vote proposal ...");
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let proposal_account_info = next_account_info(account_info_iter)?;

    payer_account_info
        .assert_signer()
        .error_log("Error: Payer account is not a signer")?;
    proposal_account_info
        .assert_owner(program_id)
        .error_log("Error: Proposal account is not owned by the program")?;
    let (_proposal_id, _proposal_bump) = proposal_account_info
        .assert_seed(
            program_id,
            &[INGL_PROPOSAL_KEY.as_ref(), &numeration.to_be_bytes()],
        )
        .error_log("failed to assert pda input for proposal_account_info")?;

    let mut governance_data = GovernanceData::decode(proposal_account_info)?;
    
    let clock_data = get_clock_data(account_info_iter, clock_is_from_account)?;
    let rent_data = get_rent_data(account_info_iter, rent_is_from_account)?;
    
    log!(log_level, 0, "about to check if proposal is ongoing");
    if governance_data.is_still_ongoing == false {
        Err(InglError::TooLate.utilize("This proposal is currently Closed"))?
    }
    log!(log_level, 0, "about to check expiration time");
    if governance_data.expiration_time < clock_data.unix_timestamp as u32 {
        Err(InglError::TooLate.utilize("This proposal is currently Expired"))?
    }
    let mut incremented_space = 0;

    for _ in 0..cnt{
        let nft_account_data_info = next_account_info(account_info_iter)?;
        let mint_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;


        verify_nft_ownership(
            payer_account_info,
            mint_account_info,
            nft_account_data_info,
            associated_token_account_info,
            program_id,
        )?;
        
        let mut nft_data = Box::new(NftData::decode(nft_account_data_info)?);
        
        log!(log_level, 0, "about to insert vote");
        governance_data.votes.insert(numeration, vote);
        nft_data.all_votes.insert(numeration, vote);
        incremented_space += 5;
        let new_space = nft_account_data_info.data.borrow().len() + 5;
        let lamports = rent_data
            .minimum_balance(new_space)
            .checked_sub(rent_data.minimum_balance(nft_account_data_info.data.borrow().len()))
            .unwrap();

        invoke(
            &system_instruction::transfer(payer_account_info.key, nft_account_data_info.key, lamports),
            &[payer_account_info.clone(), nft_account_data_info.clone()],
        )
        .error_log(
            "failed to transfer for reallaocating_nft_account_data_size @system_program invoke",
        )?;
        nft_account_data_info.realloc(new_space, false).error_log("failed to realloc nft account data account size")?;
        
        nft_data
        .serialize(&mut &mut nft_account_data_info.data.borrow_mut()[..])
        .error_log("failed to serialize into nft_account_info")?;
    }

    let new_space = proposal_account_info.data.borrow().len() + incremented_space;
    let lamports = rent_data
        .minimum_balance(new_space)
        .checked_sub(rent_data.minimum_balance(proposal_account_info.data.borrow().len()))
        .unwrap();

    invoke(
        &system_instruction::transfer(payer_account_info.key, proposal_account_info.key, lamports),
        &[payer_account_info.clone(), proposal_account_info.clone()],
    )
    .error_log(
        "failed to transfer for reallaocating_proposal_account_size @system_program invoke",
    )?;

    proposal_account_info
        .realloc(new_space, false)
        .error_log("failed to realloc gem account data account size")?;

    log!(
        log_level,
        0,
        "about to serialize into proposal_account_info"
    );
    governance_data
        .serialize(&mut &mut proposal_account_info.data.borrow_mut()[..])
        .error_log("failed to serialize into proposal_account_info")?;
    log!(log_level, 4, "Done with Vote proposal !!!");
    Ok(())
}
