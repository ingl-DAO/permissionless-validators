use crate::{
    log,
    state::{
        constants::{
            CUMMULATED_RARITY, GEM_ACCOUNT_CONST, INGL_MINT_AUTHORITY_KEY, INGL_VRF_MAX_RESULT,
        },
        NftData, UrisAccount, VrfClientState,
    },
    utils::{get_clock_data, AccountInfoHelpers, ResultExt},
};

use anchor_lang::{AnchorDeserialize, __private::bytemuck};

use borsh::BorshSerialize;

use mpl_token_metadata::state::{DataV2, Metadata, PREFIX};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use solana_program::program_pack::Pack;
use spl_associated_token_account::get_associated_token_address;
use spl_token::{error::TokenError, state::Account};
use switchboard_v2::VrfAccountData;

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

    let nft_vrf_account_info = next_account_info(account_info_iter)?;
    let nft_vrf_state_account_info = next_account_info(account_info_iter)?;

    let clock_data = get_clock_data(account_info_iter, clock_is_from_account)?;

    ingl_config_account_info
        .assert_owner(&program_id)
        .error_log("Error: Ingl config account is not owned by the program")?;

    let mut nft_data = NftData::validate(NftData::deserialize(
        &mut &nft_account_info.data.borrow()[..],
    )?)?;

    if let Some(_) = nft_data.rarity {
        Err(ProgramError::InvalidAccountData).error_log("Rarity has already been imprinted")?
    }

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

    let associated_token_account_data =
        Account::unpack(&associated_token_account_info.data.borrow())?;
    if associated_token_account_data.amount != 1 {
        Err(ProgramError::InsufficientFunds)?
    }
    if !associated_token_account_data.is_frozen() {
        Err(TokenError::AccountFrozen)?
    }

    let (mint_authority_key, mint_authority_bump) = freeze_authority_account_info
        .assert_seed(&program_id, &[INGL_MINT_AUTHORITY_KEY.as_ref()])?;

    let (_gem_pubkey, _gem_bump) =
        nft_account_info.assert_seed(&mint_account_info.key, &[GEM_ACCOUNT_CONST.as_ref()])?;

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

    let gem_metadata = Metadata::deserialize(&mut &metadata_account_info.data.borrow()[..])
        .error_log("Error: @ deserialize metadata")?;

    let vrf_account_data =
        VrfAccountData::new(nft_vrf_account_info).error_log("Failed to get vrf account data")?;
    let result_buffer = vrf_account_data
        .get_result()
        .error_log("Failed to get vrf result")?;

    if result_buffer == [0u8; 32] {
        Err(ProgramError::InvalidAccountData).error_log("Empty VRF account buffer")?
    }

    let mut vrf_state_account_data =
        VrfClientState::deserialize(&mut &nft_vrf_state_account_info.data.borrow()[..])
            .error_log("Failed to deserialize @vrf_state_account_info data")?;

    if result_buffer == vrf_state_account_data.result_buffer {
        Err(ProgramError::InvalidAccountData).error_log("Unchanged VRF result buffer")?
    }

    let uris_data = Box::new(UrisAccount::decode(&uris_account_info)?);
    let random_value = get_vrf_value(result_buffer, vrf_state_account_data.max_result, log_level);
    let seed = if clock_is_from_account {
        1
    } else {
        (random_value % CUMMULATED_RARITY as u128) as u16
    };
    let (uri, rarity) = uris_data.get_uri(seed);

    log!(log_level, 2, "Updating metadata account ...");
    invoke_signed(
        &mpl_token_metadata::instruction::update_metadata_accounts_v2(
            mpl_token_metadata_id,
            *metadata_account_info.key,
            *freeze_authority_account_info.key,
            Some(*freeze_authority_account_info.key),
            Some(DataV2 {
                uri: uri,
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

    if vrf_state_account_data.result != random_value {
        vrf_state_account_data.result_buffer = result_buffer;
        vrf_state_account_data.result = random_value;
        vrf_state_account_data.timestamp = clock_data.unix_timestamp;

        vrf_state_account_data
            .serialize(&mut &mut nft_vrf_state_account_info.data.borrow_mut()[..])
            .error_log("Failed to serialize @nft_vrf_state_account_info data")?;
    }

    log!(log_level, 4, "Imprint rarity !!!");
    Ok(())
}

fn get_vrf_value(result_buffer: [u8; 32], max_result: u64, log_level: u8) -> u128 {
    log!(log_level, 0, "Result buffer is {:?}", result_buffer);
    let value: &[u128] = bytemuck::cast_slice(&result_buffer[..]);
    log!(log_level, 0, "u128 buffer {:?}", value);
    let random_value = value[0] % max_result as u128 + 1;
    log!(
        log_level,
        0,
        "Current VRF Value [1 - {}) = {}!",
        INGL_VRF_MAX_RESULT,
        random_value
    );
    random_value
}
