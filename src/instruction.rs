use anchor_lang::system_program;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    stake::instruction::StakeInstruction,
    system_instruction, sysvar,
};

use crate::state::{constants, GovernanceType, VoteAuthorize, VoteInit, VoteState};

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
    Init {
        //Tested
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
        governance_expiration_time: u32,
        rarities: Vec<u16>,
        rarity_names: Vec<String>,
        twitter_handle: String,
        discord_invite: String,
        validator_name: String,
        collection_uri: String,
        website: String,
        default_uri: String,
    },
    Redeem {
        //Tested
        log_level: u8,
    },
    NFTWithdraw {
        //Tested
        cnt: u8,
        log_level: u8,
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
}
impl InstructionEnum {
    pub fn decode(data: &[u8]) -> Self {
        try_from_slice_unchecked(data).expect("Failed during the Desrialization of InstructionEnum")
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum RegistryInstructionEnum {
    InitConfig,
    AddProgram,
    RemovePrograms { program_count: u8 },
    Blank,
}

pub fn register_program_instruction(
    payer: Pubkey,
    program_id: Pubkey,
    storage_key: Pubkey,
) -> Instruction {
    let instr = RegistryInstructionEnum::AddProgram;
    let data = instr.try_to_vec().unwrap();
    let config_key =
        Pubkey::find_program_address(&[b"config"], &constants::program_registry::id()).0;
    Instruction {
        program_id: constants::program_registry::id(),
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(config_key, false),
            AccountMeta::new_readonly(program_id, false),
            AccountMeta::new(constants::team::id(), false),
            AccountMeta::new(storage_key, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    }
}

#[derive(Serialize, Deserialize)]
pub enum VoteInstruction {
    /// Initialize a vote account
    ///
    /// # Account references
    ///   0. `[WRITE]` Uninitialized vote account
    ///   1. `[]` Rent sysvar
    ///   2. `[]` Clock sysvar
    ///   3. `[SIGNER]` New validator identity (node_pubkey)
    InitializeAccount(VoteInit),

    ///NOT FOR USAGE:  Authorize a key to send votes or issue a withdrawal
    ///
    /// # Account references
    ///   0. `[WRITE]` Vote account to be updated with the Pubkey for authorization
    ///   1. `[]` Clock sysvar
    ///   2. `[SIGNER]` Vote or withdraw authority
    Authorize(Pubkey, VoteAuthorize),

    /// NOT FOR USAGE:   A Vote instruction with recent votes
    ///
    /// # Account references
    ///   0. `[WRITE]` Vote account to vote with
    ///   1. `[]` Slot hashes sysvar
    ///   2. `[]` Clock sysvar
    ///   3. `[SIGNER]` Vote authority
    Vote(), //Not for usage

    /// Withdraw some amount of funds
    ///
    /// # Account references
    ///   0. `[WRITE]` Vote account to withdraw from
    ///   1. `[WRITE]` Recipient account
    ///   2. `[SIGNER]` Withdraw authority
    Withdraw(u64),

    /// Update the vote account's validator identity (node_pubkey)
    ///
    /// # Account references
    ///   0. `[WRITE]` Vote account to be updated with the given authority public key
    ///   1. `[SIGNER]` New validator identity (node_pubkey)
    ///   2. `[SIGNER]` Withdraw authority
    UpdateValidatorIdentity,

    /// Update the commission for the vote account
    ///
    /// # Account references
    ///   0. `[WRITE]` Vote account to be updated
    ///   1. `[SIGNER]` Withdraw authority
    UpdateCommission(u8),

    /// A Vote instruction with recent votes
    ///
    /// # Account references
    ///   0. `[WRITE]` Vote account to vote with
    ///   1. `[]` Slot hashes sysvar
    ///   2. `[]` Clock sysvar
    ///   3. `[SIGNER]` Vote authority
    VoteSwitch(), // Two Interior fields of (Vote, Hash)
}

pub fn vote_initialize_account(vote_pubkey: &Pubkey, vote_init: &VoteInit) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*vote_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(vote_init.node_pubkey, true),
    ];

    Instruction::new_with_bincode(
        solana_program::vote::program::id(),
        &VoteInstruction::InitializeAccount(*vote_init),
        account_metas,
    )
}

pub fn vote_create_account(from_pubkey: &Pubkey, vote_pubkey: &Pubkey) -> Instruction {
    let space = VoteState::space() as u64;
    let create_ix = system_instruction::create_account(
        from_pubkey,
        vote_pubkey,
        VoteState::min_lamports(),
        space,
        &solana_program::vote::program::id(),
    );
    create_ix
}

pub fn vote_update_validator_identity(
    vote_pubkey: &Pubkey,
    authorized_withdrawer_pubkey: &Pubkey,
    node_pubkey: &Pubkey,
) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*vote_pubkey, false),
        AccountMeta::new_readonly(*node_pubkey, true),
        AccountMeta::new_readonly(*authorized_withdrawer_pubkey, true),
    ];

    Instruction::new_with_bincode(
        solana_program::vote::program::id(),
        &VoteInstruction::UpdateValidatorIdentity,
        account_metas,
    )
}

pub fn vote_withdraw(
    vote_pubkey: &Pubkey,
    authorized_withdrawer_pubkey: &Pubkey,
    lamports: u64,
    to_pubkey: &Pubkey,
) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*vote_pubkey, false),
        AccountMeta::new(*to_pubkey, false),
        AccountMeta::new_readonly(*authorized_withdrawer_pubkey, true),
    ];

    Instruction::new_with_bincode(
        solana_program::vote::program::id(),
        &VoteInstruction::Withdraw(lamports),
        account_metas,
    )
}

pub fn vote_update_commission(
    vote_pubkey: &Pubkey,
    authorized_withdrawer_pubkey: &Pubkey,
    commission: u8,
) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*vote_pubkey, false),
        AccountMeta::new_readonly(*authorized_withdrawer_pubkey, true),
    ];

    Instruction::new_with_bincode(
        solana_program::vote::program::id(),
        &VoteInstruction::UpdateCommission(commission),
        account_metas,
    )
}

pub fn vote_authorize(
    vote_pubkey: &Pubkey,
    authorized_pubkey: &Pubkey, // currently authorized
    new_authorized_pubkey: &Pubkey,
    vote_authorize: VoteAuthorize,
) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*vote_pubkey, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(*authorized_pubkey, true),
    ];

    Instruction::new_with_bincode(
        solana_program::vote::program::id(),
        &VoteInstruction::Authorize(*new_authorized_pubkey, vote_authorize),
        account_metas,
    )
}

pub fn split(
    stake_key: &Pubkey,
    pd_pool_key: &Pubkey,
    lamports: u64,
    t_withdraw_key: &Pubkey,
) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*stake_key, false),
        AccountMeta::new(*t_withdraw_key, false),
        AccountMeta::new_readonly(*pd_pool_key, true),
    ];

    Instruction::new_with_bincode(
        solana_program::stake::program::id(),
        &StakeInstruction::Split(lamports),
        account_metas,
    )
}
