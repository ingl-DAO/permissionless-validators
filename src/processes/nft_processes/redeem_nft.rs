use crate::{
    error::InglError,
    log,
    state::{constants::*, FundsLocation, GeneralData, NftData, ValidatorConfig},
    utils::{get_clock_data, verify_nft_ownership, AccountInfoHelpers, OptionExt, ResultExt},
};

use mpl_token_metadata::state::PREFIX;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction,
};
pub fn redeem_nft(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
    clock_is_from_account: bool,
) -> ProgramResult {
    log!(log_level, 4, "Redeem nft ...");
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let pd_pool_account_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let nft_account_data_info = next_account_info(account_info_iter)?;
    let nft_metadata_account_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let ingl_nft_collection_metadata_account_info = next_account_info(account_info_iter)?;
    let spl_token_program_account_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;
    let vote_account_info = next_account_info(account_info_iter)?;

    let clock_data =
        get_clock_data(account_info_iter, clock_is_from_account).error_log("sysvar_clock_data")?;

    nft_metadata_account_info
        .assert_owner(&mpl_token_metadata::id())
        .error_log("@assert nft metadata account owner")?;
    edition_account_info
        .assert_owner(&mpl_token_metadata::id())
        .error_log("@assert edition account owner")?;
    ingl_nft_collection_metadata_account_info
        .assert_owner(&mpl_token_metadata::id())
        .error_log("@assert ingl nft collection metadata account owner")?;

    spl_token_program_account_info
        .assert_key_match(&spl_token::id())
        .error_log("spl_token_program_account_info")?;

    config_account_info
        .assert_owner(program_id)
        .error_log("@assert config_account_info")?;
    general_account_info
        .assert_owner(program_id)
        .error_log("@assert general_account_info")?;
    vote_account_info
        .assert_owner(&solana_program::vote::program::id())
        .error_log("@assert vote_account_info")?;

    config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("@assert config_account_info")?;
    general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])
        .error_log("@assert general_account_info")?;
    vote_account_info
        .assert_seed(program_id, &[VOTE_ACCOUNT_KEY.as_ref()])
        .error_log("@assert vote_account_info")?;

    let (pd_pool_id, pd_pool_bump) = pd_pool_account_info
        .assert_seed(program_id, &[PD_POOL_ACCOUNT_KEY.as_ref()])
        .error_log("@assert pd pool pda")?;

    verify_nft_ownership(
        payer_account_info,
        mint_account_info,
        nft_account_data_info,
        associated_token_account_info,
        program_id,
    )?;

    let mpl_token_metadata_id = mpl_token_metadata::id();

    let (edition_key, _edition_bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            mpl_token_metadata_id.as_ref(),
            mint_account_info.key.as_ref(),
            b"edition",
        ],
        &mpl_token_metadata_id,
    );
    edition_account_info
        .assert_key_match(&edition_key)
        .error_log("Error: @edition_account_info")?;

    let metadata_seeds = &[
        PREFIX.as_ref(),
        mpl_token_metadata_id.as_ref(),
        mint_account_info.key.as_ref(),
    ];
    let (nft_metadata_key, _nft_metadata_bump) =
        Pubkey::find_program_address(metadata_seeds, &mpl_token_metadata_id);

    nft_metadata_account_info
        .assert_key_match(&nft_metadata_key)
        .error_log("Error: @meta_data_account_info")?;

    let (ingl_nft_collection_mint_key, _ingl_nft_bump) =
        Pubkey::find_program_address(&[INGL_NFT_COLLECTION_KEY.as_ref()], &program_id);

    let metadata_seeds = &[
        PREFIX.as_ref(),
        mpl_token_metadata_id.as_ref(),
        ingl_nft_collection_mint_key.as_ref(),
    ];
    let (ingl_nft_collection_metadata_key, _collection_metadata_bump) =
        ingl_nft_collection_metadata_account_info
            .assert_seed(&mpl_token_metadata_id, metadata_seeds)
            .error_log("@assert ingl nft collection metadata")?;

    let nft_data = NftData::parse(&nft_account_data_info, program_id)
        .error_log("@nft_account_info decode_unchecked validation")?;
    let config_data = Box::new(ValidatorConfig::parse(config_account_info, program_id)?);
    let general_data = Box::new(GeneralData::parse(general_account_info, program_id)?);

    match nft_data.funds_location {
        FundsLocation::Undelegated => {}
        _ => Err(InglError::InvalidFundsLocation.utilize("nft_account_redeem_nft"))?,
    };

    let now = clock_data.unix_timestamp as u32;
    let nft_age = (now.checked_sub(nft_data.date_created)).unwrap();

    let mut redeem_fees: u64 = 0;
    if now > general_data.last_feeless_redemption_date
        && nft_age < config_data.redemption_fee_duration
    {
        redeem_fees = config_data.get_redeem_fee(nft_age);
        log!(
            log_level,
            1,
            "Now: {}, spent_time: {}, redeem_fees: {}",
            now,
            nft_age,
            redeem_fees
        );

        invoke_signed(
            &system_instruction::transfer(&pd_pool_id, vote_account_info.key, redeem_fees),
            &[pd_pool_account_info.clone(), vote_account_info.clone()],
            &[&[PD_POOL_ACCOUNT_KEY.as_ref(), &[pd_pool_bump]]],
        )?;
        log!(log_level, 2, "Transfered Funds to vote account!!!");
    }

    log!(log_level, 2, "Transfering funds to user ...");
    invoke_signed(
        &system_instruction::transfer(
            &pd_pool_id,
            payer_account_info.key,
            config_data
                .unit_backing
                .checked_sub(redeem_fees)
                .error_log("Error @ Redeem Fees Sub from NFT Backing lamports")?,
        ),
        &[pd_pool_account_info.clone(), payer_account_info.clone()],
        &[&[PD_POOL_ACCOUNT_KEY.as_ref(), &[pd_pool_bump]]],
    )
    .error_log("@invoke system_intruction transfer")?;
    log!(log_level, 2, "Transfered Funds to user!!!");

    log!(log_level, 2, "Burn the nft ...");
    invoke(
        &mpl_token_metadata::instruction::burn_nft(
            mpl_token_metadata_id,
            nft_metadata_key,
            *payer_account_info.key,
            *mint_account_info.key,
            *associated_token_account_info.key,
            edition_key,
            spl_token::id(),
            Some(ingl_nft_collection_metadata_key),
        ),
        &[
            nft_metadata_account_info.clone(),
            payer_account_info.clone(),
            mint_account_info.clone(),
            associated_token_account_info.clone(),
            edition_account_info.clone(),
            spl_token_program_account_info.clone(),
            ingl_nft_collection_metadata_account_info.clone(),
        ],
    )
    .error_log("@invoke mpl_token burn nft")?;
    log!(log_level, 2, "Burned the nft!!!");

    let dest_starting_lamports = payer_account_info.lamports();
    **payer_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(nft_account_data_info.lamports())
        .error_log("Error @ payer_lamports increase")?;
    **nft_account_data_info.lamports.borrow_mut() = 0;

    let mut payer_nft_data = nft_account_data_info.data.borrow_mut();
    payer_nft_data.fill(0);

    log!(log_level, 4, "Redeemed nft !!!");
    Ok(())
}
