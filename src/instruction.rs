use anchor_lang::system_program;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::state::{constants, GovernanceType};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct InitArgs {
    pub log_level: u8,
    pub init_commission: u8,
    pub max_primary_stake: u64,
    pub nft_holders_share: u8,
    pub initial_redemption_fee: u8,
    pub is_validator_id_switchable: bool,
    pub unit_backing: u64,
    pub redemption_fee_duration: u32,
    pub proposal_quorum: u8,
    pub creator_royalties: u16,
    pub governance_expiration_time: u32,
    pub rarities: Vec<u16>,
    pub rarity_names: Vec<String>,
    pub twitter_handle: String,
    pub discord_invite: String,
    pub validator_name: String,
    pub collection_uri: String,
    pub website: String,
    pub default_uri: String,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum InstructionEnum {
    MintNft {
        // Tested
        log_level: u8,
    },
    ImprintRarity {
        //Tested
        log_level: u8,
    },
    Init(InitArgs),
    Redeem {
        //Tested
        log_level: u8,
    },
    NFTWithdraw {
        //Tested
        log_level: u8,
        cnt: u8,
    },
    ProcessRewards {
        //Tested
        log_level: u8,
    },
    InitRebalance {
        //Tested
        log_level: u8,
    },
    FinalizeRebalance {
        //Tested
        log_level: u8,
    },
    UploadUris {
        //Tested
        uris: Vec<String>,
        rarity: u8,
        log_level: u8,
    },
    ResetUris {
        //Tested
        log_level: u8,
    },
    UnDelegateNFT {
        //Tested
        log_level: u8,
    },
    DelegateNFT {
        //Tested
        log_level: u8,
    },
    CreateVoteAccount {
        //Tested
        log_level: u8,
    },
    InitGovernance {
        //Tested
        governance_type: GovernanceType,
        title: String,
        description: String,
        log_level: u8,
    },
    VoteGovernance {
        //Tested
        numeration: u32,
        vote: bool,
        cnt: u8,
        log_level: u8,
    },
    FinalizeGovernance {
        //Untested
        numeration: u32,
        log_level: u8,
    },
    ExecuteGovernance {
        //Untested
        numeration: u32,
        log_level: u8,
    },
    InjectTestingData {
        //Untested
        num_mints: u8,
        log_level: u8,
    },
    FractionalizeExisting(InitArgs),
}

impl InstructionEnum {
    pub fn decode(data: &[u8]) -> Self {
        try_from_slice_unchecked(data).expect("Failed during the Desrialization of InstructionEnum")
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum RegistryInstructionEnum {
    InitConfig,
    AddProgram { name: String },
    RemovePrograms { program_count: u8 },
    Reset,
    Blank,
}

pub fn register_program_instruction(
    payer: Pubkey,
    program_id: Pubkey,
    name: String,
) -> Instruction {
    let instr = RegistryInstructionEnum::AddProgram { name };
    let data = instr.try_to_vec().unwrap();
    let config_key =
        Pubkey::find_program_address(&[b"config"], &constants::program_registry::id()).0;
    let (name_storage_key, _name_storage_bump) =
        Pubkey::find_program_address(&[b"name_storage"], &constants::program_registry::id());
    let (storage_key, _storage_bump) =
        Pubkey::find_program_address(&[b"storage"], &constants::program_registry::id());

    let accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(config_key, false),
        AccountMeta::new_readonly(program_id, false),
        AccountMeta::new(constants::team::id(), false),
        AccountMeta::new(storage_key, false),
        AccountMeta::new(name_storage_key, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    Instruction {
        program_id: constants::program_registry::id(),
        accounts: accounts,
        data,
    }
}
