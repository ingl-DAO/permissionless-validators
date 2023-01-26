use crate::{
    instruction::register_program_instruction,
    log,
    state::{constants::*, GeneralData, UrisAccount, ValidatorConfig},
    utils::{get_rent_data_from_account, AccountInfoHelpers, OptionExt, ResultExt},
};
use borsh::{BorshSerialize};
use mpl_token_metadata::state::{Creator, PREFIX};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction, system_program, sysvar,
};

pub fn process_init(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    log_level: u8,
    init_commission: u8,
    max_primary_stake: u64,
    nft_holders_share: u8,
    initial_redemption_fee: u8,
    is_validator_id_switchable: bool,
    unit_backing: u64,
    redemption_fee_duration: u32,
    proposal_quorum: u8,
    creator_royalties: u16,
    rarities: Vec<u16>,
    rarity_names: Vec<String>,
    governance_expiration_time: u32,
    twitter_handle: String,
    discord_invite: String,
    validator_name: String,
    collection_uri: String,
    website: String,
    default_uri: String,
) -> ProgramResult {
    log!(log_level, 4, "Init Process Started");
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let config_account_info = next_account_info(account_info_iter)?;
    let general_account_info = next_account_info(account_info_iter)?;
    let uris_account_info = next_account_info(account_info_iter)?;
    let rent_account_info = next_account_info(account_info_iter)?;
    let validator_account_info = next_account_info(account_info_iter)?;
    let collection_holder_account_info = next_account_info(account_info_iter)?;
    let collection_mint_account_info = next_account_info(account_info_iter)?;
    let mint_authority_account_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let collection_metadata_account_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let spl_token_program_account_info = next_account_info(account_info_iter)?;
    let system_program_account_info = next_account_info(account_info_iter)?;

    let registry_program_config_account = next_account_info(account_info_iter)?;
    let this_program_account_info = next_account_info(account_info_iter)?;
    let team_account_info = next_account_info(account_info_iter)?;
    let storage_account_info = next_account_info(account_info_iter)?;

    let rent_data = get_rent_data_from_account(rent_account_info)?;

    log!(log_level, 0, "Collected Main Accounts succesfully ... ");

    // payer_account_info
    //     .assert_signer()
    //     .error_log("Error @ Payer's Signature Assertion")?;
    // payer_account_info
    //     .assert_key_match(&initializer::id())
    //     .error_log("Error @ Payer's Key Match Assertion")?;
    let (config_key, config_bump) = config_account_info
        .assert_seed(program_id, &[INGL_CONFIG_SEED])
        .error_log("Error @ Config Account Seed Assertion")?;
    let (general_account_key, general_account_bump) = general_account_info
        .assert_seed(program_id, &[GENERAL_ACCOUNT_SEED])
        .error_log("Error @ General Account Seed Assertion")?;
    let (uri_account_key, uri_account_bump) = uris_account_info
        .assert_seed(program_id, &[URIS_ACCOUNT_SEED])
        .error_log("Error @ Uris Account Seed Assertion")?;
    system_program_account_info
        .assert_key_match(&system_program::id())
        .error_log("Error @ system_program_account_info Assertion")?;
    spl_token_program_account_info
        .assert_key_match(&spl_token::id())
        .error_log("Error @ spl_token_program_account_info Assertion")?;
    registry_program_config_account
        .assert_owner(&program_registry::id())
        .error_log("Error @ registry_program_config_account Assertion")?;
    this_program_account_info
        .assert_key_match(program_id)
        .error_log("Error @ this_program_account_info Assertion")?;
    team_account_info
        .assert_key_match(&team::id())
        .error_log("Error @ team_account_info Assertion")?;

    let create_collection_accounts = &[
        payer_account_info.clone(),
        collection_holder_account_info.clone(),
        collection_mint_account_info.clone(),
        mint_authority_account_info.clone(),
        associated_token_account_info.clone(),
        collection_metadata_account_info.clone(),
        edition_account_info.clone(),
        spl_token_program_account_info.clone(),
        rent_account_info.clone(),
        system_program_account_info.clone(),
    ];
    mint_collection(
        program_id,
        create_collection_accounts,
        collection_uri.clone(),
        validator_name.clone(),
        log_level,
    )
    .error_log("Error: @mint_collection creation")?;

    let mut rarity_name_space = 0;
    for i in rarity_names.iter() {
        rarity_name_space += i.len() + 4;
    }

    let uris_account_creation_size = 16 + rarities.len() * 2 + rarity_name_space;
    let uris_account_creation_lamports = rent_data.minimum_balance(uris_account_creation_size);
    log!(log_level, 2, "Creating Uris Account ... ");
    invoke_signed(
        &system_instruction::create_account(
            payer_account_info.key,
            &uri_account_key,
            uris_account_creation_lamports,
            uris_account_creation_size as u64,
            program_id,
        ),
        &[payer_account_info.clone(), uris_account_info.clone()],
        &[&[URIS_ACCOUNT_SEED, &[uri_account_bump]]],
    )?;
    log!(log_level, 2, "Created Uris Account !!!");

    let config_data = ValidatorConfig::new(
        is_validator_id_switchable,
        max_primary_stake,
        nft_holders_share,
        initial_redemption_fee,
        unit_backing,
        redemption_fee_duration,
        proposal_quorum,
        creator_royalties,
        init_commission,
        *validator_account_info.key,
        governance_expiration_time,
        default_uri,
        validator_name,
        twitter_handle,
        discord_invite,
        website,
    )?;

    let general_data = GeneralData::default();

    let config_account_creation_size = config_data.get_space();
    let config_account_creation_lamports = rent_data.minimum_balance(config_account_creation_size);
    log!(log_level, 2, "Creating Config Account ... ");
    invoke_signed(
        &system_instruction::create_account(
            payer_account_info.key,
            &config_key,
            config_account_creation_lamports,
            config_account_creation_size as u64,
            program_id,
        ),
        &[payer_account_info.clone(), config_account_info.clone()],
        &[&[INGL_CONFIG_SEED, &[config_bump]]],
    )?;
    log!(log_level, 2, "Created Config Account ... ");

    let general_account_creation_size = general_data.get_space();
    let general_account_creation_lamports =
        rent_data.minimum_balance(general_account_creation_size);
    log!(log_level, 2, "Creating General Account ... ");
    invoke_signed(
        &system_instruction::create_account(
            payer_account_info.key,
            &general_account_key,
            general_account_creation_lamports,
            general_account_creation_size as u64,
            program_id,
        ),
        &[payer_account_info.clone(), general_account_info.clone()],
        &[&[GENERAL_ACCOUNT_SEED, &[general_account_bump]]],
    )?;
    log!(log_level, 2, "Created General Account ... ");

    let uri_data = UrisAccount::new(rarities, rarity_names)?;

    log!(log_level, 0, "Created Main Data succesfully ... ");

    config_data
        .serialize(&mut &mut config_account_info.data.borrow_mut()[..])
        .error_log("Error @ Config Account Data Serialization")?;
    general_data
        .serialize(&mut &mut general_account_info.data.borrow_mut()[..])
        .error_log("Error @ General Account Data Serialization")?;
    uri_data
        .serialize(&mut &mut uris_account_info.data.borrow_mut()[..])
        .error_log("Error @ Uris Account Data Serialization")?;

    log!(log_level, 2, "Initing Program Registration ... ");
    invoke(
        &register_program_instruction(
            *payer_account_info.key,
            *program_id,
            *storage_account_info.key,
        ),
        &[
            payer_account_info.clone(),
            registry_program_config_account.clone(),
            this_program_account_info.clone(),
            team_account_info.clone(),
            storage_account_info.clone(),
            system_program_account_info.clone(),
        ],
    )?;

    log!(log_level, 4, "Initialization completed !!!");
    Ok(())
}

// Mint ingl GEMs collection
fn mint_collection(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    collection_uri: String,
    validator_name: String,
    log_level: u8,
) -> ProgramResult {
    log!(log_level, 4, "Creating collection...");
    let account_info_iter = &mut accounts.iter();

    let payer_account_info = next_account_info(account_info_iter)?;
    let collection_holder_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let mint_authority_account_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let collection_metadata_account_info = next_account_info(account_info_iter)?;
    let ingl_edition_account_info = next_account_info(account_info_iter)?;
    let spl_token_program_account_info = next_account_info(account_info_iter)?;
    let sysvar_rent_account_info = next_account_info(account_info_iter)?;
    let system_program_account_info = next_account_info(account_info_iter)?;

    payer_account_info
        .assert_signer()
        .error_log("Payer signature required")?;

    system_program_account_info
        .assert_key_match(&system_program::id())
        .error_log("sent system_program_account_address is dissimilar from expected one")?;
    spl_token_program_account_info
        .assert_key_match(&spl_token::id())
        .error_log("sent spl_token_program_account_address is dissimilar from expected one")?;
    sysvar_rent_account_info
        .assert_key_match(&sysvar::rent::id())
        .error_log("sent sysvar_rent_account_address is dissimilar from expected one")?;

    let (_ingl_gem_collection_key, ingl_gem_bump) = mint_account_info
        .assert_seed(program_id, &[INGL_NFT_COLLECTION_KEY.as_ref()])
        .error_log("failed to assert pda input for mint_account_info")?;
    let (mint_authority_key, mint_authority_bump) = mint_authority_account_info
        .assert_seed(program_id, &[INGL_MINT_AUTHORITY_KEY.as_ref()])
        .error_log("failed to assert pda input to mint_authority_account_info")?;

    let mut creators = Vec::new();
    creators.push(Creator {
        address: mint_authority_key,
        verified: true,
        share: 100,
    });

    let mpl_token_metadata_id = mpl_token_metadata::id();
    let metadata_seeds = &[
        PREFIX.as_ref(),
        mpl_token_metadata_id.as_ref(),
        mint_account_info.key.as_ref(),
    ];

    let (gem_metadata_key, _gem_metadata_bump) = collection_metadata_account_info
        .assert_seed(&mpl_token_metadata_id, metadata_seeds)
        .error_log("sent gem_meta_data_account_address is dissimilar from expected one")?;

    let (collection_holder_key, _chk_bump) = collection_holder_account_info
        .assert_seed(program_id, &[COLLECTION_HOLDER_KEY.as_ref()])
        .error_log("failed to assert pda input to collection_holder_account_info")?;

    let collection_associated_pubkey = spl_associated_token_account::get_associated_token_address(
        &collection_holder_key,
        mint_account_info.key,
    );
    associated_token_account_info
        .assert_key_match(&collection_associated_pubkey)
        .error_log("sent associated_token_account_address is dissimilar from expected one")?;

    let edition_metadata_seeds = &[
        b"metadata",
        mpl_token_metadata_id.as_ref(),
        mint_account_info.key.as_ref(),
        b"edition",
    ];
    let (collection_edition_key, _edition_bump) = ingl_edition_account_info
        .assert_seed(&mpl_token_metadata_id, edition_metadata_seeds)
        .error_log("sent edition_account_address is dissimilar from expected one")?;

    let space = 82;
    let rent_lamports =
        get_rent_data_from_account(sysvar_rent_account_info)?.minimum_balance(space);

    log!(log_level, 2, "Creating mint account ...");
    invoke_signed(
        &system_instruction::create_account(
            payer_account_info.key,
            mint_account_info.key,
            rent_lamports,
            space as u64,
            spl_token_program_account_info.key,
        ),
        &[payer_account_info.clone(), mint_account_info.clone()],
        &[&[INGL_NFT_COLLECTION_KEY.as_ref(), &[ingl_gem_bump]]],
    )
    .error_log("failed to create mint_account @system_program invoke")?;

    log!(log_level, 2, "Initializing mint account ...");
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
    .error_log("failed to initialize mint_account @spl_program invoke")?;

    log!(log_level, 2, "Creating associated token account ...");
    invoke(
        &spl_associated_token_account::instruction::create_associated_token_account(
            payer_account_info.key,
            collection_holder_account_info.key,
            mint_account_info.key,
            &spl_token::id()
        ),
        &[
            payer_account_info.clone(),
            associated_token_account_info.clone(),
            collection_holder_account_info.clone(),
            mint_account_info.clone(),
            system_program_account_info.clone(),
            spl_token_program_account_info.clone(),
        ],
    ).error_log("failed to create associated token on collection_holder_account @spl_associated_program invoke")?;

    log!(log_level, 2, "Minting new collection ...");
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
    .error_log("failed to mint collection @spl_program invoke")?;

    log!(log_level, 2, "Creating metaplex nft account v3 ...");
    invoke_signed(
        &mpl_token_metadata::instruction::create_metadata_accounts_v3(
            mpl_token_metadata_id,
            gem_metadata_key,
            *mint_account_info.key,
            *mint_authority_account_info.key,
            *payer_account_info.key,
            *mint_authority_account_info.key,
            validator_name.clone(),
            format!(
                "{}_U",
                validator_name
                    .get(
                        0..(if validator_name.len() > 8 {
                            8
                        } else {
                            validator_name.len()
                        })
                    )
                    .error_log("error determining collection symbol")?
            ), //TODO prompt user for symbol
            collection_uri,
            Some(creators),
            300,
            true,
            true,
            None,
            None,
            None,
        ),
        &[
            collection_metadata_account_info.clone(),
            mint_account_info.clone(),
            mint_authority_account_info.clone(),
            payer_account_info.clone(),
            mint_authority_account_info.clone(),
            system_program_account_info.clone(),
            sysvar_rent_account_info.clone(),
        ],
        &[&[INGL_MINT_AUTHORITY_KEY.as_ref(), &[mint_authority_bump]]],
    )
    .error_log("failed to create metadata_account @metaplex_program invoke")?;

    log!(log_level, 2, "Creating Master edition account ...");
    invoke_signed(
        &mpl_token_metadata::instruction::create_master_edition_v3(
            mpl_token_metadata_id,
            collection_edition_key,
            *mint_account_info.key,
            mint_authority_key,
            mint_authority_key,
            gem_metadata_key,
            *payer_account_info.key,
            Some(0),
        ),
        &[
            ingl_edition_account_info.clone(),
            mint_account_info.clone(),
            mint_authority_account_info.clone(),
            mint_authority_account_info.clone(),
            payer_account_info.clone(),
            collection_metadata_account_info.clone(),
            spl_token_program_account_info.clone(),
            system_program_account_info.clone(),
            sysvar_rent_account_info.clone(),
        ],
        &[&[INGL_MINT_AUTHORITY_KEY.as_ref(), &[mint_authority_bump]]],
    )
    .error_log("failed to create master_edition_account @metaplex_program invoke")?;

    log!(log_level, 4, "Collection created!!!");
    Ok(())
}
