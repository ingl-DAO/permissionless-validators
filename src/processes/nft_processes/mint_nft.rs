use crate::{
    log,
    state::{constants::*, FundsLocation, GeneralData, NftData, UrisAccount, ValidatorConfig},
    utils::{get_clock_data, get_rent_data_from_account, AccountInfoHelpers, ResultExt},
};

use borsh::BorshSerialize;
use mpl_token_metadata::{
    self as metaplex,
    state::{Collection, Creator, PREFIX},
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction, system_program,
    sysvar::{self},
};

use spl_associated_token_account::{get_associated_token_address, *};

pub fn process_mint_nft(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
    clock_is_from_account: bool,
) -> ProgramResult {
    log!(log_level, 4, "Initiated nft Minting ...");
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let mint_authority_account_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let spl_token_program_account_info = next_account_info(account_info_iter)?;
    let sysvar_rent_account_info = next_account_info(account_info_iter)?;
    let system_program_account_info = next_account_info(account_info_iter)?;
    let nft_metadata_account_info = next_account_info(account_info_iter)?;
    let minting_pool_account_info = next_account_info(account_info_iter)?;
    let nft_account_info = next_account_info(account_info_iter)?;
    let ingl_edition_account_info = next_account_info(account_info_iter)?;
    let nft_edition_account_info = next_account_info(account_info_iter)?;
    let ingl_collection_mint_info = next_account_info(account_info_iter)?;
    let ingl_collection_account_info = next_account_info(account_info_iter)?;
    let ingl_config_account_info = next_account_info(account_info_iter)?;
    let uris_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;
    let clock = get_clock_data(account_info_iter, clock_is_from_account)?;
    let rent_data = get_rent_data_from_account(sysvar_rent_account_info)?;

    log!(log_level, 0, "Done with Main account Collection ...");

    payer_account_info
        .assert_signer()
        .error_log("Error @ Payer signer assertion")?;

    ingl_edition_account_info
        .assert_owner(&metaplex::id())
        .error_log("Error @ ingl_edition_account ownership assertion")?;
    ingl_collection_account_info
        .assert_owner(&metaplex::id())
        .error_log("Error @ ingl_collection_account ownership assertion")?;
    ingl_collection_mint_info
        .assert_owner(&spl_token::id())
        .error_log("Error @ ingl_collection_mint ownership assertion")?;

    system_program_account_info
        .assert_key_match(&system_program::id())
        .error_log("Error: @system_program_account_info")?;
    spl_token_program_account_info
        .assert_key_match(&spl_token::id())
        .error_log("Error: @spl_token_program_account_info")?;
    sysvar_rent_account_info
        .assert_key_match(&sysvar::rent::id())
        .error_log("Error: @sysvar_rent_account_info assertion")?;

    let (nft_account_pubkey, nft_account_bump) = nft_account_info
        .assert_seed(
            program_id,
            &[NFT_ACCOUNT_CONST.as_ref(), mint_account_info.key.as_ref()],
        )
        .error_log("Error @ nft_account_info pda assertion")?;
    let (pd_pool_id, _pd_pool_bump) = minting_pool_account_info
        .assert_seed(program_id, &[PD_POOL_ACCOUNT_KEY.as_ref()])
        .error_log("Error @ minting_pool_account_info pda assertion")?;
    let (mint_authority_key, mint_authority_bump) = mint_authority_account_info
        .assert_seed(program_id, &[INGL_MINT_AUTHORITY_KEY.as_ref()])
        .error_log("Error @ mint_authority_account_info pda assertion")?;
    let (ingl_nft_collection_key, _ingl_nft_bump) = ingl_collection_mint_info
        .assert_seed(program_id, &[INGL_NFT_COLLECTION_KEY.as_ref()])
        .error_log("Error @ ingl_collection_mint_info pda assertion")?;

    let (_ingl_config_key, _ingl_config_bump) = ingl_config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED.as_ref()])
        .error_log("Error @ ingl_config_account_info pda assertion")?;
    ingl_collection_account_info
        .assert_owner(program_id)
        .error_log("Error @ ingl_collection_account_info ownership assertion")?;

    let (_uris_account_key, _uris_account_bump) = uris_account_info
        .assert_seed(program_id, &[URIS_ACCOUNT_SEED.as_ref()])
        .error_log("Error @ uris_account_info pda assertion")?;
    uris_account_info
        .assert_owner(program_id)
        .error_log("Error @ uris_account_info ownership assertion")?;

    let (_general_account_key, _general_account_bump) = general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])
        .error_log("Error @ general_account_info pda assertion")?;
    general_account_info
        .assert_owner(program_id)
        .error_log("Error @ general_account_info ownership assertion")?;

    let config_data = Box::new(ValidatorConfig::decode(&ingl_config_account_info)?);
    let uris_data = Box::new(UrisAccount::decode(&uris_account_info)?);
    let mut general_data = Box::new(GeneralData::decode(&general_account_info)?);

    let mpl_token_metadata_id = mpl_token_metadata::id();
    let metadata_seeds = &[
        PREFIX.as_bytes(),
        mpl_token_metadata_id.as_ref(),
        mint_account_info.key.as_ref(),
    ];
    let (nft_metadata_key, _nft_metadata_bump) = nft_metadata_account_info
        .assert_seed(&mpl_token_metadata_id, metadata_seeds)
        .error_log("Error @ nft_metadata_account_info pda assertion")?;

    associated_token_account_info
        .assert_key_match(&get_associated_token_address(
            payer_account_info.key,
            mint_account_info.key,
        ))
        .error_log("Error: @associated_token_account_info")?;

    let collection_metadata_seeds = &[
        PREFIX.as_ref(),
        mpl_token_metadata_id.as_ref(),
        ingl_nft_collection_key.as_ref(),
    ];
    let (collection_metadata_key, _collection_metadata_bump) = ingl_collection_account_info
        .assert_seed(&mpl_token_metadata_id, collection_metadata_seeds)
        .error_log("Error @ collection_metadata_key pda assertion")?;

    let (nft_edition_key, _edition_bump) = nft_edition_account_info.assert_seed(
        &mpl_token_metadata_id,
        &[
            b"metadata",
            mpl_token_metadata_id.as_ref(),
            mint_account_info.key.as_ref(),
            b"edition",
        ],
    )?;
    let (ingl_collection_edition_key, _ingl_edition_bump) = ingl_edition_account_info
        .assert_seed(
            &mpl_token_metadata_id,
            &[
                b"metadata",
                mpl_token_metadata_id.as_ref(),
                ingl_nft_collection_key.as_ref(),
                b"edition",
            ],
        )
        .error_log("Error @ ingl_edition_account_info pda assertion")?;

    log!(log_level, 0, "Done with main assertions");

    // Getting timestamp
    let current_timestamp = clock.unix_timestamp as u32;

    let space = 130;
    let rent_lamports = rent_data.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            payer_account_info.key,
            &nft_account_pubkey,
            rent_lamports,
            space as u64,
            program_id,
        ),
        &[payer_account_info.clone(), nft_account_info.clone()],
        &[&[
            NFT_ACCOUNT_CONST.as_ref(),
            mint_account_info.key.as_ref(),
            &[nft_account_bump],
        ]],
    )
    .error_log("Error @ nft_account_info creation")?;

    let space = 82;
    let rent_lamports = rent_data.minimum_balance(space);

    let mint_cost = config_data.unit_backing;

    general_data.mint_numeration += 1;
    general_data.total_delegated += 1;

    if general_data.dealloced_count >= 1 {
        general_data.dealloced_count -= 1;
    } else {
        general_data.pending_delegation_count += 1;
    }

    log!(log_level, 2, "transfer the mint cost to the minting pool");
    //tranfer token from one account to an other
    invoke(
        &system_instruction::transfer(payer_account_info.key, &pd_pool_id, mint_cost),
        &[
            payer_account_info.clone(),
            minting_pool_account_info.clone(),
        ],
    )
    .error_log("Error @ minting_pool_account_info transfer")?;

    log!(log_level, 2, "create the mint account");
    invoke(
        &system_instruction::create_account(
            payer_account_info.key,
            mint_account_info.key,
            rent_lamports,
            space as u64,
            spl_token_program_account_info.key,
        ),
        &[payer_account_info.clone(), mint_account_info.clone()],
    )
    .error_log("Error @ mint_account_info creation")?;
    log!(log_level, 2, "initialize the mint account");
    invoke(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_account_info.key,
            &mint_authority_key,
            Some(&mint_authority_key),
            0,
        )?,
        &[mint_account_info.clone(), sysvar_rent_account_info.clone()],
    )
    .error_log("Error @ mint_account_info initialization")?;
    log!(log_level, 2, "create mint associated token account");
    invoke(
        &spl_associated_token_account::instruction::create_associated_token_account(
            payer_account_info.key,
            payer_account_info.key,
            mint_account_info.key,
        ),
        &[
            payer_account_info.clone(),
            associated_token_account_info.clone(),
            payer_account_info.clone(),
            mint_account_info.clone(),
            system_program_account_info.clone(),
            spl_token_program_account_info.clone(),
        ],
    )
    .error_log("Error @ associated_token_account_info creation")?;

    log!(log_level, 2, "Mint new token");
    invoke_signed(
        &spl_token::instruction::mint_to(
            spl_token_program_account_info.key,
            mint_account_info.key,
            associated_token_account_info.key,
            &mint_authority_key,
            &[],
            1,
        )?,
        &[
            mint_account_info.clone(),
            associated_token_account_info.clone(),
            mint_authority_account_info.clone(),
        ],
        &[&[INGL_MINT_AUTHORITY_KEY.as_ref(), &[mint_authority_bump]]],
    )
    .error_log("Error @ minting token")?;

    let mut creators = Vec::new();
    creators.push(Creator {
        address: mint_authority_key,
        verified: true,
        share: 100,
    });

    let seed = if clock_is_from_account {
        1
    } else {
        panic!("Haven't yet written the seed source")
    };

    let (uri, rarity) = uris_data.get_uri(seed);

    log!(log_level, 2, "starting metadata creation");
    invoke_signed(
        &mpl_token_metadata::instruction::create_metadata_accounts_v3(
            mpl_token_metadata_id,
            nft_metadata_key,
            *mint_account_info.key,
            *mint_authority_account_info.key,
            *payer_account_info.key,
            *mint_authority_account_info.key,
            format!(
                "{} #{}",
                config_data.validator_name, &general_data.mint_numeration
            ),
            format!(
                "{} #{}",
                config_data.validator_name, &general_data.mint_numeration
            ),
            uri,
            Some(creators),
            300,
            true,
            true,
            Some(Collection {
                verified: false,
                key: ingl_nft_collection_key,
            }),
            None,
            None,
        ),
        &[
            nft_metadata_account_info.clone(),
            mint_account_info.clone(),
            mint_authority_account_info.clone(),
            payer_account_info.clone(),
            mint_authority_account_info.clone(),
            system_program_account_info.clone(),
            sysvar_rent_account_info.clone(),
        ],
        &[&[INGL_MINT_AUTHORITY_KEY.as_ref(), &[mint_authority_bump]]],
    )
    .error_log("Error @ nft_metadata_account_info creation")?;

    log!(log_level, 2, "verifying collection");
    invoke_signed(
        &mpl_token_metadata::instruction::verify_collection(
            mpl_token_metadata_id,
            nft_metadata_key,
            mint_authority_key,
            *payer_account_info.key,
            ingl_nft_collection_key,
            collection_metadata_key,
            ingl_collection_edition_key,
            None,
        ),
        &[
            nft_metadata_account_info.clone(),
            mint_authority_account_info.clone(),
            payer_account_info.clone(),
            ingl_collection_mint_info.clone(),
            ingl_collection_account_info.clone(),
            ingl_edition_account_info.clone(),
        ],
        &[&[INGL_MINT_AUTHORITY_KEY.as_ref(), &[mint_authority_bump]]],
    )
    .error_log("Error @ collection verification")?;

    log!(log_level, 2, "Creating master Edition account...");
    invoke_signed(
        &mpl_token_metadata::instruction::create_master_edition_v3(
            mpl_token_metadata_id,
            nft_edition_key,
            *mint_account_info.key,
            mint_authority_key,
            mint_authority_key,
            nft_metadata_key,
            *payer_account_info.key,
            None,
        ),
        &[
            nft_edition_account_info.clone(),
            mint_account_info.clone(),
            mint_authority_account_info.clone(),
            mint_authority_account_info.clone(),
            payer_account_info.clone(),
            nft_metadata_account_info.clone(),
            spl_token_program_account_info.clone(),
            system_program_account_info.clone(),
            sysvar_rent_account_info.clone(),
        ],
        &[&[INGL_MINT_AUTHORITY_KEY.as_ref(), &[mint_authority_bump]]],
    )
    .error_log("Error @ master Edition creation")?;
    log!(log_level, 2, "Delegate mint authority to metaplex...");
    invoke(
        &spl_token::instruction::approve(
            &spl_token::id(),
            associated_token_account_info.key,
            &mint_authority_key,
            payer_account_info.key,
            &[],
            1,
        )?,
        &[
            associated_token_account_info.clone(),
            mint_authority_account_info.clone(),
            payer_account_info.clone(),
        ],
    )
    .error_log("Error @ mint authority delegation")?;

    log!(
        log_level,
        2,
        "updating update_primary_sale_happened_via_token"
    );
    invoke(
        &mpl_token_metadata::instruction::update_primary_sale_happened_via_token(
            mpl_token_metadata::id(),
            nft_metadata_key,
            *payer_account_info.key,
            *associated_token_account_info.key,
        ),
        &[
            nft_metadata_account_info.clone(),
            payer_account_info.clone(),
            associated_token_account_info.clone(),
        ],
    )
    .error_log("Error @ update_primary_sale_happened_via_token")?;

    let nft_account_data = NftData {
        validation_phrase: NFT_DATA_VAL_PHRASE,
        date_created: current_timestamp,
        numeration: general_data.mint_numeration,
        rarity: rarity,
        funds_location: FundsLocation::Delegated,
        all_withdraws: Vec::new(),
        all_votes: Vec::new(),
    };
    nft_account_data
        .serialize(&mut &mut nft_account_info.data.borrow_mut()[..])
        .error_log("Error @ nft_account_data serialization")?;

    general_data
        .serialize(&mut &mut general_account_info.data.borrow_mut()[..])
        .error_log("Error @ general_data serialization")?;
    log!(log_level, 4, "nft account created!!!");

    Ok(())
}