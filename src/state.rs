#![allow(unused_parens)]
use crate::{
    colored_log,
    error::InglError,
    utils::{assert_program_owned, AccountInfoHelpers, ResultExt},
};
use borsh::{BorshDeserialize, BorshSerialize};
use ingl_macros::Validate;
use serde::{Deserialize, Serialize};
use solana_program::{
    account_info::AccountInfo,
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    native_token::LAMPORTS_PER_SOL,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use crate::state::LogColors::*;
pub const LOG_LEVEL: u8 = 5;

pub mod constants {
    pub const INGL_CONFIG_VAL_PHRASE: u32 = 739_215_648;
    pub const URIS_ACCOUNT_VAL_PHRASE: u32 = 382_916_043;
    pub const GENERAL_ACCOUNT_VAL_PHRASE: u32 = 836_438_471;
    pub const NFT_DATA_VAL_PHRASE: u32 = 271_832_912;

    pub const INGL_CONFIG_SEED: &[u8] = b"ingl_config";
    pub const URIS_ACCOUNT_SEED: &[u8] = b"uris_account";
    pub const GENERAL_ACCOUNT_SEED: &[u8] = b"general_account";
    pub const INGL_NFT_COLLECTION_KEY: &[u8] = b"ingl_nft_collection";
    pub const INGL_MINT_AUTHORITY_KEY: &[u8] = b"ingl_mint_authority";
    pub const COLLECTION_HOLDER_KEY: &[u8] = b"collection_holder";
    pub const VOTE_ACCOUNT_KEY: &[u8] = b"vote_account";
    pub const AUTHORIZED_WITHDRAWER_KEY: &[u8] = b"authorized_withdrawer";
    pub const STAKE_ACCOUNT_KEY: &[u8] = b"stake_account";
    pub const PD_POOL_ACCOUNT_KEY: &[u8] = b"pd_pool_account";
    pub const NFT_ACCOUNT_CONST: &[u8] = b"nft_account";

    pub mod initializer {
        solana_program::declare_id!("62uPowNXr22WPw7XghajJkWMBJ2fnv1oGthxqHYYPHie");
    }

    pub mod config {
        solana_program::declare_id!("Config1111111111111111111111111111111111111");
    }
}

#[derive(BorshSerialize, BorshDeserialize, Validate)]
#[validation_phrase(crate::state::constants::INGL_CONFIG_VAL_PHRASE)]
///Creation Size: 15 + 4 + 4 + 4 + len(discord_invite) + len(twitter_handle) + len(validator_name)
pub struct ValidatorConfig {
    pub validation_phrase: u32,
    pub is_validator_id_switchable: bool,
    pub max_primary_stake: u64,
    pub nft_holders_share: u8,
    pub initial_redemption_fee: u8,
    pub unit_backing: u64,
    pub redemption_fee_duration: u32,
    pub program_upgrade_threshold: u8,
    pub creator_royalties: u16,
    pub commission: u8,
    pub validator_id: Pubkey,
    pub validator_name: String,
    pub twitter_handle: String,
    pub discord_invite: String,
}

impl ValidatorConfig {
    pub fn validate_data(&self) -> ProgramResult {
        if !self.is_validator_id_switchable && self.initial_redemption_fee != 0 {
            Err(InglError::InvalidConfigData
                .utilize("Validator id must be switchable if there exists any redemption fee"))?
        }
        if self.redemption_fee_duration > 86400 * 365 * 2 {
            Err(InglError::InvalidConfigData
                .utilize("Redemption fee duration must be less than 2 years"))?
        }
        if self.nft_holders_share > 100 {
            Err(InglError::InvalidConfigData.utilize("NFT holders share must be less than 100%"))?
        }
        if self.initial_redemption_fee > 25 {
            Err(InglError::InvalidConfigData
                .utilize("Initial redemption fee must be less than 25%"))?
        }
        if self.unit_backing < LAMPORTS_PER_SOL {
            Err(InglError::InvalidConfigData.utilize("Unit backing must be greater than 1 Sol"))?
        }
        if self.max_primary_stake < LAMPORTS_PER_SOL {
            Err(InglError::InvalidConfigData
                .utilize("Max primary stake must be greater than 1 Sol"))?
        }
        if self.validation_phrase != constants::INGL_CONFIG_VAL_PHRASE {
            Err(InglError::InvalidConfigData.utilize("Validation phrase is incorrect"))?
        }
        if self.program_upgrade_threshold > 100 {
            Err(InglError::InvalidConfigData
                .utilize("Program upgrade threshold must be less than 100%"))?
        }

        if self.program_upgrade_threshold < 65 {
            Err(InglError::InvalidConfigData
                .utilize("Program upgrade threshold must be less than 65%"))?
        }
        if self.creator_royalties > 500 {
            Err(InglError::InvalidConfigData.utilize("Creator royalties must be less than 5%"))?
        }
        Ok(())
    }

    pub fn new(
        is_validator_id_switchable: bool,
        max_primary_stake: u64,
        nft_holders_share: u8,
        initial_redemption_fee: u8,
        unit_backing: u64,
        redemption_fee_duration: u32,
        program_upgrade_threshold: u8,
        creator_royalties: u16,
        commission: u8,
        validator_id: Pubkey,
        validator_name: String,
        twitter_handle: String,
        discord_invite: String,
    ) -> Result<Self, ProgramError> {
        let i = Self {
            validation_phrase: constants::INGL_CONFIG_VAL_PHRASE,
            is_validator_id_switchable,
            max_primary_stake,
            nft_holders_share,
            initial_redemption_fee,
            unit_backing,
            redemption_fee_duration,
            program_upgrade_threshold,
            creator_royalties,
            commission,
            validator_id,
            validator_name,
            twitter_handle,
            discord_invite,
        };
        i.validate_data()
            .error_log("Error @ Config Data Validation")?;
        Ok(i)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Validate)]
#[validation_phrase(crate::state::constants::URIS_ACCOUNT_VAL_PHRASE)]
///Creation Size: 8
pub struct UrisAccount {
    pub validation_phrase: u32,
    ///This vector is used to define rarity of NFTs.
    /// i.e. if there are 3 rarities,  and the first rarity is 60%, the second is 30% and the third is 10%
    /// then the vector will be [6000, 9000, 10000]
    pub rarities: Vec<u16>,
    pub uris: Vec<Vec<String>>,
}
impl UrisAccount {
    pub fn new(rarities: Vec<u16>) -> Result<Self, ProgramError> {
        if rarities.iter().sum::<u16>() != 10000 {
            Err(InglError::InvalidUrisAccountData.utilize("Rarities must sum to 10000"))?
        }
        let mut new_rarities = Vec::new();
        for i in rarities {
            if i == 0 {
                Err(InglError::InvalidUrisAccountData.utilize("Rarities must be greater than 0"))?
            }
            if new_rarities.len() == 0 {
                new_rarities.push(i);
            } else {
                new_rarities.push(i + new_rarities[new_rarities.len() - 1]);
            }
        }

        let i = Self {
            validation_phrase: constants::URIS_ACCOUNT_VAL_PHRASE,
            rarities: new_rarities,
            uris: Vec::new(),
        };
        i.validate_data()
            .error_log("Error @ Uris Account Data Validation")?;
        Ok(i)
    }

    pub fn validate_data(&self) -> ProgramResult {
        if self.validation_phrase != constants::URIS_ACCOUNT_VAL_PHRASE {
            Err(InglError::InvalidUrisAccountData.utilize("Validation phrase is incorrect"))?
        }
        if self.rarities.len() == 0 {
            Err(InglError::InvalidUrisAccountData.utilize("Rarities vector is empty"))?
        }
        if *self.rarities.last().unwrap() != 10000 {
            Err(InglError::InvalidUrisAccountData.utilize("Rarities must sum to 10000"))?
        }
        Ok(())
    }

    pub fn set_uri(&mut self, rarity: u8, uris: Vec<String>) -> Result<usize, ProgramError> {
        if rarity as usize > self.rarities.len() {
            Err(InglError::InvalidUrisAccountData.utilize("Rarity is out of bounds"))?
        }
        if uris.len() == 0 {
            Err(InglError::InvalidUrisAccountData.utilize("Uris vector is empty"))?
        }
        let mut space = 0;
        for i in uris.iter() {
            space += i.len() + 4;
        }
        if self.uris.len() == rarity.into() {
            self.uris.push(uris);
            space += 4;
        } else {
            self.uris[rarity as usize].extend(uris);
        }
        Ok(space)
    }

    pub fn get_uri(&self, seed: u16) -> (String, u8) {
        let ind = self.rarities.iter().position(|x| *x > seed).unwrap();
        (
            self.uris[ind][seed as usize % self.uris[ind].len()].clone(),
            ind as u8,
        )
    }
}

#[derive(BorshDeserialize, Copy, Clone, PartialEq, Debug, BorshSerialize)]
///Creation Size: 24 bytes.
/// This Stores the Cummulative of rewards for a specific vote account for the epoch the process_rewards instruction was run.
pub struct VoteReward {
    /// This is the epoch the reward was earned.
    pub epoch_number: u64,
    /// This is the amount of rewards earned.
    pub total_reward: u64,
    /// This is the total primary stake of the vote account.
    pub total_stake: u64,
}

#[derive(BorshDeserialize, Copy, Clone, PartialEq, Debug, BorshSerialize)]
/// Creation Size: 17 bytes.
pub struct RebalancingData {
    /// This is the reward that was earned by the validator in the form of staking reward since the last rebalancing whose total reward was > 1Sol.
    pub pending_validator_rewards: u64,
    /// This is the total Reward that was earned by the validator in the form of staking reward since the last rebalancing whose total rewards < 1Sol.
    pub unclaimed_validator_rewards: u64,
    /// This tells us whether the rebalancing process is active or not.
    pub is_rebalancing_active: bool,
}

impl Default for RebalancingData {
    fn default() -> Self {
        Self {
            pending_validator_rewards: 0,
            unclaimed_validator_rewards: 0,
            is_rebalancing_active: false,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Validate)]
#[validation_phrase(crate::state::constants::GENERAL_ACCOUNT_VAL_PHRASE)]
///Creation Size: 90 Bytes
pub struct GeneralData {
    pub validation_phrase: u32,
    pub mint_numeration: u32,
    pub pending_delegation_count: u32,
    pub dealloced_count: u32,
    pub total_delegated: u32,
    pub last_withdraw_epoch: u64,
    pub last_total_staked: u64,
    pub is_t_stake_initialized: bool,
    pub rebalancing_data: RebalancingData,
    pub vote_rewards: Vec<VoteReward>,
}

impl Default for GeneralData {
    fn default() -> Self {
        Self {
            validation_phrase: constants::GENERAL_ACCOUNT_VAL_PHRASE,
            mint_numeration: 0,
            pending_delegation_count: 0,
            dealloced_count: 0,
            total_delegated: 0,
            last_withdraw_epoch: 0,
            last_total_staked: 0,
            is_t_stake_initialized: false,
            rebalancing_data: RebalancingData::default(),
            vote_rewards: Vec::new(),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum FundsLocation {
    Delegated,
    Undelegated,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum GovernanceVote {
    Yes,
    No,
}

#[derive(BorshSerialize, BorshDeserialize, Validate)]
#[validation_phrase(crate::state::constants::NFT_DATA_VAL_PHRASE)]
//Creation Size:
pub struct NftData {
    pub validation_phrase: u32,
    pub rarity: u8,
    pub funds_location: FundsLocation,
    pub numeration: u32,
    pub date_created: u32,
    pub all_withdraws: Vec<u64>,
    pub all_votes: Vec<GovernanceVote>,
}

pub enum LogColors {
    Red,
    Green,
    Blue,
    Blank,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct VoteInit {
    pub node_pubkey: Pubkey,
    pub authorized_voter: Pubkey,
    pub authorized_withdrawer: Pubkey,
    pub commission: u8,
}

pub struct VoteState {}
impl VoteState {
    pub fn space() -> usize {
        3731
    }
    pub fn min_lamports() -> u64 {
        Rent::get().unwrap().minimum_balance(Self::space())
    }
}
