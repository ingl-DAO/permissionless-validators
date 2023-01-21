use crate::{
    error::InglError,
    log,
    state::{
        constants::{INGL_MINT_AUTHORITY_KEY, NETWORK, NFT_ACCOUNT_CONST},
        get_feeds, Network, NftData, UrisAccount,
    },
    utils::{get_clock_data, AccountInfoHelpers, OptionExt, ResultExt},
};

use anchor_lang::AnchorDeserialize;

use borsh::BorshSerialize;

use mpl_token_metadata::state::{DataV2, Metadata, PREFIX};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    blake3::hash,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use solana_program::program_pack::Pack;
use spl_associated_token_account::get_associated_token_address;
use spl_token::{error::TokenError, state::Account};
use switchboard_v2::{
    AggregatorHistoryBuffer, AggregatorHistoryRow, SWITCHBOARD_PROGRAM_ID, SWITCHBOARD_V2_DEVNET,
};

pub fn process_imprint_rarity(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
    clock_is_from_account: bool,
) -> ProgramResult {
    log!(log_level, 4, "Imprint rarity...");
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let nft_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let freeze_authority_account_info = next_account_info(account_info_iter)?;
    let metadata_account_info = next_account_info(account_info_iter)?;
    let nft_edition_account_info = next_account_info(account_info_iter)?;
    let ingl_config_account_info = next_account_info(account_info_iter)?;
    let uris_account_info = next_account_info(account_info_iter)?;

    let clock_data = get_clock_data(account_info_iter, clock_is_from_account)?;

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
    associated_token_account_info
        .assert_owner(&spl_token::id())
        .error_log("Error: @associated_token_account_info ownership")?;

    associated_token_account_info
        .assert_key_match(&get_associated_token_address(
            payer_account_info.key,
            mint_account_info.key,
        ))
        .error_log("associated_token_account_info")?;

    ingl_config_account_info
        .assert_owner(&program_id)
        .error_log("Error: Ingl config account is not owned by the program")?;

    let mut nft_data = NftData::validate(
        NftData::deserialize(&mut &nft_account_info.data.borrow()[..])
            .error_log("Error: Error desirializing NFT account data")?,
    )
    .error_log("Error: Invalid NFT Account")?;

    if (clock_data.unix_timestamp as u32)
        < nft_data
            .rarity_seed_time
            .error_log("Error: Rarity seed time can't be None")?
    {
        Err(InglError::TooEarly.utilize("imprint_rarity"))?
    }
    if let Some(_) = nft_data.rarity {
        Err(ProgramError::InvalidAccountData).error_log("Rarity has already been imprinted")?
    }

    let associated_token_account_data =
        Account::unpack(&associated_token_account_info.data.borrow())?;
    if associated_token_account_data.amount != 1 {
        Err(ProgramError::InsufficientFunds)?
    }
    if !associated_token_account_data.is_frozen() {
        Err(TokenError::AccountFrozen)?
    }

    let (mint_authority_key, mint_authority_bump) = freeze_authority_account_info
        .assert_seed(&program_id, &[INGL_MINT_AUTHORITY_KEY.as_ref()])
        .error_log("@mint_authority_accoun_info")?;
    let (_nft_pubkey, _nft_bump) = nft_account_info
        .assert_seed(&mint_account_info.key, &[NFT_ACCOUNT_CONST.as_ref()])
        .error_log("Error: @nft_account_info")?;

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
            freeze_authority_account_info.clone(),
            associated_token_account_info.clone(),
            nft_edition_account_info.clone(),
            mint_account_info.clone(),
        ],
        &[&[INGL_MINT_AUTHORITY_KEY.as_ref(), &[mint_authority_bump]]],
    )
    .error_log("Error: @ thawing the token account")?;
    log!(log_level, 2, "Token account thawed !!!");

    let gem_metadata = Metadata::deserialize(&mut &metadata_account_info.data.borrow()[..])
        .error_log("Error: @ deserialize metadata")?;

    let mut rarity_hash_bytes = Vec::new();

    let interested_network = NETWORK;
    let history_feeds_pubkeys = get_feeds(&interested_network);
    log!(log_level, 0, "starting history feeds loop");
    for cnt in 0..history_feeds_pubkeys.len() {
        let history_feed_account_info = next_account_info(account_info_iter)?;
        match interested_network {
            Network::LocalTest => {
                history_feed_account_info
                    .assert_owner(&SWITCHBOARD_V2_DEVNET)
                    .error_log("@history_feed")?;
            }
            _ => {
                history_feed_account_info
                    .assert_owner(&SWITCHBOARD_PROGRAM_ID)
                    .error_log("@history_feed")?;
            }
        }
        if &history_feeds_pubkeys[cnt] != history_feed_account_info.key {
            Err(InglError::InvalidHistoryBufferKeys
                .utilize(&format!("a problem with history buffer key No: {}", cnt)))?
        }

        let history_feed = AggregatorHistoryBuffer::new(history_feed_account_info)
            .error_log(&format!("Error getting history feed No: {}", cnt))?;
        let AggregatorHistoryRow {
            value,
            timestamp: _,
        } = history_feed
            .lower_bound(
                nft_data
                    .rarity_seed_time
                    .error_log("Error: rarity seed time can't be None")? as i64,
            )
            .error_log("Error @ getting AggregatorHistoryRow")?;

        let history_feed_price = value.mantissa as u128;
        rarity_hash_bytes.extend(&history_feed_price.to_be_bytes());
    }
    log!(log_level, 0, "finished history feeds loop");
    rarity_hash_bytes.extend(&program_id.to_bytes());
    rarity_hash_bytes.extend(&mint_account_info.key.to_bytes());

    let uris_data = Box::new(UrisAccount::parse(&uris_account_info, program_id)?);
    let seed = if clock_is_from_account {
        1
    } else {
        get_rarity_seed(rarity_hash_bytes.clone())
    };
    let (nft_rarity_uri, rarity) = uris_data.get_uri(seed);

    log!(log_level, 2, "Updating metadata account ...");
    invoke_signed(
        &mpl_token_metadata::instruction::update_metadata_accounts_v2(
            mpl_token_metadata_id,
            *metadata_account_info.key,
            *freeze_authority_account_info.key,
            Some(*freeze_authority_account_info.key),
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
            freeze_authority_account_info.clone(),
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

fn get_rarity_seed(seed_bytes: Vec<u8>) -> u16 {
    let rarity_hash_bytes = hash(&seed_bytes).to_bytes();

    let mut byte_sum: u64 = 0;
    for byte in rarity_hash_bytes {
        byte_sum = byte_sum + (byte as u64).pow(3);
    }

    (byte_sum % 10000) as u16
}
