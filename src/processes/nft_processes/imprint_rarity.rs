use crate::{
    error::InglError,
    log,
    state::{constants::INGL_MINT_AUTHORITY_KEY, NftData, UrisAccount},
    utils::{get_clock_data, verify_nft_ownership, AccountInfoHelpers, OptionExt, ResultExt},
};

use arrayref::array_ref;
use borsh::{BorshDeserialize, BorshSerialize};

use mpl_token_metadata::state::{DataV2, Metadata, PREFIX};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

pub fn process_imprint_rarity(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
    clock_is_from_account: bool,
) -> ProgramResult {
    log!(log_level, 4, "Imprint nft rarity ...");
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let nft_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let mint_authority_account_info = next_account_info(account_info_iter)?;
    let metadata_account_info = next_account_info(account_info_iter)?;
    let nft_edition_account_info = next_account_info(account_info_iter)?;
    let ingl_config_account_info = next_account_info(account_info_iter)?;
    let uris_account_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    let recent_blockhashes_account_info = next_account_info(account_info_iter)?;

    log!(log_level, 0, "Done retrieving accounts infos");
    let clock_data = get_clock_data(account_info_iter, clock_is_from_account)?;
    log!(log_level, 0, "verifying nft accounts infos ....");
    verify_nft_ownership(
        payer_account_info,
        mint_account_info,
        nft_account_info,
        associated_token_account_info,
        program_id,
    )?;
    log!(log_level, 0, "Done verifying nft accounts infos");
    payer_account_info
        .assert_signer()
        .error_log("Error: @payer_account_info")?;
    nft_account_info
        .assert_owner(&program_id)
        .error_log("Error: @gem_account_info ownership")?;
    mint_account_info
        .assert_owner(&spl_token::id())
        .error_log("Error: @mint_account_info ownership")?;
    nft_edition_account_info
        .assert_owner(&mpl_token_metadata::id())
        .error_log("Error: @nft_edition_account_info ownership")?;
    metadata_account_info
        .assert_owner(&mpl_token_metadata::id())
        .error_log("Error: @metadata_account_info ownership")?;
    recent_blockhashes_account_info
        .assert_owner(&solana_program::sysvar::id())
        .error_log("Error: @recent_blockhashes_account_info ownership")?;
    recent_blockhashes_account_info
        .assert_key_match(&sysvar::slot_hashes::ID)
        .error_log("Error: @recent_blockhashes_account_info key")?;
    ingl_config_account_info
        .assert_owner(&program_id)
        .error_log("Error: Ingl config account is not owned by the program")?;

    let mut nft_data = NftData::validate(
        NftData::deserialize(&mut &nft_account_info.data.borrow()[..])
            .error_log("Error: Error desirializing NFT account data")?,
    )
    .error_log("Error: Invalid NFT Account")?;

    log!(log_level, 0, "Checking deserialized data...");
    if (clock_data.slot as u64)
        < nft_data
            .rarity_seed_slot
            .error_log("Error: Rarity seed slot can't be None")?
    {
        Err(InglError::TooEarly.utilize("imprint_rarity"))?
    }
    if let Some(_) = nft_data.rarity {
        Err(ProgramError::InvalidAccountData).error_log("Rarity has already been imprinted")?
    }
    log!(log_level, 2, "Done Checking deserialized data !!!");

    let (mint_authority_key, mint_authority_bump) = mint_authority_account_info
        .assert_seed(&program_id, &[INGL_MINT_AUTHORITY_KEY.as_ref()])
        .error_log("@mint_authority_accoun_info")?;
    let mpl_token_metadata_id = mpl_token_metadata::id();
    let (nft_edition_key, _nft_edition_bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            mpl_token_metadata_id.as_ref(),
            mint_account_info.key.as_ref(),
            b"edition",
        ],
        &mpl_token_metadata_id,
    );
    nft_edition_account_info
        .assert_key_match(&nft_edition_key)
        .error_log("Error: @edition_account_info")?;

    let mpl_token_metadata_id = mpl_token_metadata::id();
    let metadata_seeds = &[
        PREFIX.as_bytes(),
        mpl_token_metadata_id.as_ref(),
        mint_account_info.key.as_ref(),
    ];
    let (nft_metadata_key, _nft_metadata_bump) =
        Pubkey::find_program_address(metadata_seeds, &mpl_token_metadata::id());

    metadata_account_info
        .assert_key_match(&nft_metadata_key)
        .error_log("Error: @meta_data_account_info")?;

    log!(log_level, 2, "Thawing the token account ...");
    invoke_signed(
        &mpl_token_metadata::instruction::thaw_delegated_account(
            mpl_token_metadata_id,
            mint_authority_key,
            *associated_token_account_info.key,
            nft_edition_key,
            *mint_account_info.key,
        ),
        &[
            mint_authority_account_info.clone(),
            associated_token_account_info.clone(),
            nft_edition_account_info.clone(),
            mint_account_info.clone(),
            token_program_account_info.clone(),
        ],
        &[&[INGL_MINT_AUTHORITY_KEY.as_ref(), &[mint_authority_bump]]],
    )
    .error_log("Error: @ thawing the token account")?;
    log!(log_level, 2, "Token account thawed !!!");

    let gem_metadata = Metadata::deserialize(&mut &metadata_account_info.data.borrow()[..])
        .error_log("Error: @ deserialize metadata")?;

    let recent_blockhashes_data = recent_blockhashes_account_info.data.borrow(); //TODO: ensure to only use the blockhash for the rarity seed slot, and fail use the common rarity if the slot is too old.
    let most_recent = array_ref![recent_blockhashes_data, 12, 8];

    let seed = ((u64::from_le_bytes(*most_recent).saturating_sub(clock_data.unix_timestamp as u64))
        % 10000) as u16;

    let uris_data = Box::new(UrisAccount::parse(&uris_account_info, program_id)?);
    let (nft_rarity_uri, rarity) = uris_data.get_uri(seed);

    log!(log_level, 2, "Updating metadata account ...");
    invoke_signed(
        &mpl_token_metadata::instruction::update_metadata_accounts_v2(
            mpl_token_metadata_id,
            *metadata_account_info.key,
            *mint_authority_account_info.key,
            Some(*mint_authority_account_info.key),
            Some(DataV2 {
                uri: nft_rarity_uri,
                uses: gem_metadata.uses,
                name: gem_metadata.data.name,
                symbol: gem_metadata.data.symbol,
                collection: gem_metadata.collection,
                creators: gem_metadata.data.creators,
                seller_fee_basis_points: gem_metadata.data.seller_fee_basis_points,
            }),
            Some(gem_metadata.primary_sale_happened),
            Some(gem_metadata.is_mutable),
        ),
        &[
            metadata_account_info.clone(),
            mint_authority_account_info.clone(),
        ],
        &[&[INGL_MINT_AUTHORITY_KEY.as_ref(), &[mint_authority_bump]]],
    )
    .error_log("Error: @ updating metadata account")?;
    log!(log_level, 2, "Metadata account updated!!!");

    nft_data.rarity = Some(rarity);
    nft_data
        .serialize(&mut &mut nft_account_info.data.borrow_mut()[..])
        .error_log("Failed to serialize @nft_account_info data")?;

    log!(log_level, 4, "Imprint rarity !!!");
    Ok(())
}
