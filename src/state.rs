#![allow(unused_parens)]
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    str::FromStr,
};

use crate::{
    colored_log,
    error::InglError,
    utils::{AccountInfoHelpers, ResultExt},
};
use bincode::deserialize;
use borsh::{BorshDeserialize, BorshSerialize};
use ingl_macros::Validate;
use serde_derive::{Deserialize, Serialize};
use solana_program::{
    account_info::AccountInfo,
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    native_token::LAMPORTS_PER_SOL,
    program_error::ProgramError,
    pubkey::Pubkey,
    slot_history::Slot,
    stake::state::StakeState,
    stake_history::Epoch,
    sysvar::{rent::Rent, Sysvar},
};

use crate::state::LogColors::*;

use self::constants::CUMMULATED_RARITY;
pub const LOG_LEVEL: u8 = 5;

pub enum Network {
    Devnet,
    Mainnet,
    LocalTest,
}

pub mod constants {
    use super::Network;

    pub const CUMMULATED_RARITY: u16 = 10000;
    pub const INGL_VRF_MAX_RESULT: u64 = 10000;
    pub const INGL_CONFIG_VAL_PHRASE: u32 = 739_215_648;
    pub const URIS_ACCOUNT_VAL_PHRASE: u32 = 382_916_043;
    pub const GENERAL_ACCOUNT_VAL_PHRASE: u32 = 836_438_471;
    pub const NFT_DATA_VAL_PHRASE: u32 = 271_832_912;
    pub const GOVERNANCE_DATA_VAL_PHRASE: u32 = 675_549_872;

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
    pub const INGL_PROGRAM_AUTHORITY_KEY: &[u8] = b"ingl_program_authority";
    pub const INGL_PROPOSAL_KEY: &[u8] = b"ingl_proposal";
    pub const VRF_STATE_KEY: &[u8] = b"ingl_vrf_state_key";
    pub const VALIDATOR_ID_SEED: &[u8] = b"validator_ID___________________";
    pub const T_STAKE_ACCOUNT_KEY: &[u8] = b"t_stake_account_key";
    pub const T_WITHDRAW_KEY: &[u8] = b"t_withdraw_key";

    pub const FEELESS_REDEMPTION_PERIOD: u32 = 86400 * 30; // 1 month
    pub const GOVERNANCE_EXECUTION_THRESHOLD: f64 = 4.0 / 5.0; // 80%
    pub const GOVERNANCE_SAFETY_LEEWAY: u32 = 86400 * 30; // 1 month

    pub const DEV_NUM_FEEDS: usize = 20;
    pub const DEV_PRICE_TIME_INTERVAL: u8 = 5;
    pub const DEV_FEEDS: [&str; DEV_NUM_FEEDS] = [
        "9ATrvi6epR5hVYtwNs7BB7VCiYnd4WM7e8MfafWpfiXC", //BTC
        "7LLvRhMs73FqcLkA8jvEE1AM2mYZXTmqfUv8GAEurymx", //SOL
        "6fhxFvPocWapZ5Wa2miDnrX2jYRFKvFqYnX11GGkBo2f", //ETH
        "DR6PqK15tD21MEGSLmDpXwLA7Fw47kwtdZeUMdT7vd7L", //BNB
        "HPRYVJQ3DcTqszvorS4gCwbJvvNeWMgaCCoF3Lj3sAgC", //ADA
        "2qcLzR7FatMnfCbiB9BdhGsd6SxDgEqWq7xkD62n3xoT", //BCH
        "Bux82YCH8DgqFAQTKBxuQHDp3cud5AhD1Kibhjadz22D", //SBR
        "9gGvxPErkRubNj1vKE19smLa4Kp89kkzMVyA6TMvmKEZ", //ZEC
        "3WNhN4RJwRui4R3k1S9agGzyMZkCwKQkWjoEHbDeAF8J", //LUNA
        "CNzjdKHfXqyAeGd2APpzvwLcuPACrFdHb3k6SLsod6Ao", //TRX
        "6cBTHY4HQ4PABmhUqVLT4n4bNpmZAi2br5VnqTQoVRUo", //SUSHI
        "GRGMtrTszsoNzjqwTxsvkHVAPq5Snju2UzaAws5KBPed", //DOGE
        "C9CeLP5B4Lqq7cFppRBUZjt6hrvd99YR3Sk4EPPuAoAC", //LTC
        "FReW6u9YPpGQNaeEHNkVqA4KGA2WzbcT87NThwFb7fwm", //XLM
        "GEp5pZFjFPqn1teMmx9sLPyADf9N9aQsRn9TE17PwmmL", //LINK
        "Fd3UQMqmKCA6SNf6To97PdC2H3EfzYWR5bxr5CBYuFiy", //DOT
        "EQHf8ueSzJUPELF6yZkyGfwjbLsDmMwFrAYehmC15b6c", //XMR
        "C5x5W7BHVY61ULtWQ3qkP7kpE6zHViWd4AHpKDuAywPw", //SRM
        "HnbpTLbdv78hkVCDBZ52o5E6bkqtsZp4tUXBd2E8Sw9x", //PORT
        "EbpMMgMkC4Jt2oipUBc2GPL4XQo5uxKT8NpF8NEZWvqL", //PAI
    ];
    pub const MAIN_NUM_FEEDS: usize = 2;
    pub const MAIN_PRICE_TIME_INTERVAL: u8 = 20;
    pub const MAIN_FEEDS: [&str; MAIN_NUM_FEEDS] = [
        "8SXvChNYFhRq4EZuZvnhjrB3jJRQCv4k3P4W6hesH3Ee", //BTC
        "E3cqnoFvTeKKNsGmC8YitpMjo2E39hwfoyt2Aiem7dCb", //SOL
    ];
    pub const LOCALTEST_NUM_FEEDS: usize = 2;
    pub const LOCALTEST_PRICE_TIME_INTERVAL: u8 = 20;
    pub const LOCALTEST_FEEDS: [&str; MAIN_NUM_FEEDS] = [
        "9ATrvi6epR5hVYtwNs7BB7VCiYnd4WM7e8MfafWpfiXC", //BTC
        "7LLvRhMs73FqcLkA8jvEE1AM2mYZXTmqfUv8GAEurymx", //SOL
    ];

    pub const NETWORK: Network = Network::Devnet;

    pub mod initializer {
        solana_program::declare_id!("62uPowNXr22WPw7XghajJkWMBJ2fnv1oGthxqHYYPHie");
    }

    pub mod program_registry {
        solana_program::declare_id!("38pfsot7kCZkrttx1THEDXEz4JJXmCCcaDoDieRtVuy5");
    }

    pub mod team {
        pub const TEAM_SHARE: u64 = 10;
        solana_program::declare_id!("Team111111111111111111111111111111111111111");
    }
}

pub fn get_min_stake_account_lamports() -> u64 {
    LAMPORTS_PER_SOL + Rent::default().minimum_balance(std::mem::size_of::<StakeState>() as usize)
}

#[derive(BorshSerialize, BorshDeserialize, Validate)]
#[validation_phrase(crate::state::constants::INGL_CONFIG_VAL_PHRASE)]
pub struct ValidatorConfig {
    pub validation_phrase: u32,
    pub is_validator_id_switchable: bool,
    pub max_primary_stake: u64,
    pub nft_holders_share: u8,
    pub initial_redemption_fee: u8,
    pub unit_backing: u64,
    pub redemption_fee_duration: u32,
    pub proposal_quorum: u8,
    pub creator_royalties: u16,
    pub commission: u8,
    pub validator_id: Pubkey,
    pub vote_account: Pubkey,
    pub governance_expiration_time: u32,
    pub default_uri: String,
    pub validator_name: String,
    pub twitter_handle: String,
    pub discord_invite: String,
    pub website: String,
}

impl ValidatorConfig {
    pub fn get_space(&self) -> usize {
        // 4 + 1 + 8 + 1 + 1 + 8 + 4 + 1 + 2 + 1 + 32 + 32 + 4 + (self.collection_uri.len() + 4) + (self.validator_name.len() + 4) + (self.twitter_handle.len() + 4) + (self.discord_invite.len() + 4) + (self.website.len() + 4)
        // 4 + 1 + 8 + 1 + 1 + 8 + 4 + 1 + 2 + 1 + 32 + 32 + 4 + 4 + 4 + 4 + 4 + 4  = 119
        119 + self.default_uri.len()
            + self.validator_name.len()
            + self.twitter_handle.len()
            + self.discord_invite.len()
            + self.website.len()
    }

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
        if self.nft_holders_share < 50 {
            Err(InglError::InvalidConfigData.utilize("NFT holders share must be greater than 50%"))?
        }
        if self.initial_redemption_fee > 25 {
            Err(InglError::InvalidConfigData
                .utilize("Initial redemption fee must be less than 25%"))?
        }
        if self.unit_backing < get_min_stake_account_lamports() {
            Err(InglError::InvalidConfigData.utilize("Unit backing must be greater than 1.03 Sol"))?
        }
        if self.max_primary_stake < self.unit_backing {
            Err(InglError::InvalidConfigData
                .utilize("Max primary stake must be greater unit backing."))?
        }
        if self.validation_phrase != constants::INGL_CONFIG_VAL_PHRASE {
            Err(InglError::InvalidConfigData.utilize("Validation phrase is incorrect"))?
        }
        if self.proposal_quorum > 100 {
            Err(InglError::InvalidConfigData
                .utilize("Program upgrade threshold must be less than 100%"))?
        }
        if self.proposal_quorum < 65 {
            Err(InglError::InvalidConfigData
                .utilize("Program upgrade threshold must be less than 65%"))?
        }
        if self.creator_royalties > 200 {
            Err(InglError::InvalidConfigData.utilize("Creator royalties must be less than 2%"))?
        }
        if self.commission > 100 {
            Err(InglError::InvalidConfigData.utilize("Commission must be less than 100%"))?
        }
        if self.validator_name.len() > 32 {
            Err(InglError::InvalidConfigData
                .utilize("Validator name must be less than 32 characters"))?
        }
        if self.twitter_handle.len() > 32 {
            Err(InglError::InvalidConfigData
                .utilize("Twitter handle must be less than 32 characters"))?
        }
        if self.discord_invite.len() > 32 {
            Err(InglError::InvalidConfigData
                .utilize("Discord invite must be less than 32 characters"))?
        }
        if self.website.len() > 64 {
            Err(InglError::InvalidConfigData.utilize("Website must be less than 32 characters"))?
        }
        if self.governance_expiration_time < 86400 * 35 {
            Err(InglError::InvalidConfigData
                .utilize("Governance expiration time must be greater than 35 days"))?
        }
        if self.governance_expiration_time > 86400 * 365 {
            Err(InglError::InvalidConfigData
                .utilize("Governance expiration time must be less than 1 year"))?
        }
        if self.default_uri.len() > 75 {
            Err(InglError::InvalidConfigData
                .utilize("Collection URI must be less than 64 characters"))?
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
        proposal_quorum: u8,
        creator_royalties: u16,
        commission: u8,
        validator_id: Pubkey,
        vote_account: Pubkey,
        governance_expiration_time: u32,
        default_uri: String,
        validator_name: String,
        twitter_handle: String,
        discord_invite: String,
        website: String,
    ) -> Result<Self, ProgramError> {
        let i = Self {
            validation_phrase: constants::INGL_CONFIG_VAL_PHRASE,
            is_validator_id_switchable,
            max_primary_stake,
            nft_holders_share,
            initial_redemption_fee,
            unit_backing,
            redemption_fee_duration,
            proposal_quorum,
            creator_royalties,
            commission,
            validator_id,
            vote_account,
            governance_expiration_time,
            default_uri,
            validator_name,
            twitter_handle,
            discord_invite,
            website,
        };
        i.validate_data()
            .error_log("Error @ Config Data Validation")?;
        Ok(i)
    }
    pub fn get_redeem_fee(&self, age: u32) -> u64 {
        if age > self.redemption_fee_duration {
            return 0;
        }

        ((((self.initial_redemption_fee as u64).pow(2)
            * ((self.redemption_fee_duration as u64).pow(2) - (age as u64).pow(2)))
            as f64)
            .sqrt() as u64)
            .checked_div(self.redemption_fee_duration as u64)
            .unwrap()
            .checked_mul(self.unit_backing)
            .unwrap()
            .checked_div(100)
            .unwrap()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Validate)]
#[validation_phrase(crate::state::constants::URIS_ACCOUNT_VAL_PHRASE)]
pub struct UrisAccount {
    pub validation_phrase: u32,
    ///This vector is used to define rarity of NFTs.
    /// i.e. if there are 3 rarities,  and the first rarity is 60%, the second is 30% and the third is 10%
    /// then the vector will be [6000, 9000, 10000]
    pub rarities: Vec<u16>,
    pub rarity_names: Vec<String>,
    pub uris: Vec<Vec<String>>,
}
impl UrisAccount {
    pub fn new(rarities: Vec<u16>, names: Vec<String>) -> Result<Self, ProgramError> {
        if rarities.iter().sum::<u16>() != CUMMULATED_RARITY {
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

        let uri_account = Self {
            validation_phrase: constants::URIS_ACCOUNT_VAL_PHRASE,
            rarity_names: names,
            rarities: new_rarities,
            uris: Vec::new(),
        };
        uri_account
            .validate_data()
            .error_log("Error @ Uris Account Data Validation")?;
        Ok(uri_account)
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
        if self.rarity_names.len() != self.rarities.len() {
            Err(InglError::InvalidUrisAccountData
                .utilize("Rarity names vector length must be equal to rarities vector length"))?
        }
        Ok(())
    }

    pub fn set_uri(&mut self, rarity: u8, uris: Vec<String>) -> Result<(), ProgramError> {
        msg!("Rarity: {}", rarity);
        if rarity as usize > self.rarities.len() {
            Err(InglError::InvalidUrisAccountData.utilize("Rarity is out of bounds"))?
        }

        if self.uris.len() == rarity as usize {
            self.uris.push(uris);
        } else {
            self.uris[rarity as usize].extend(uris);
        }
        Ok(())
    }

    pub fn get_uri(&self, mut seed: u16) -> (String, u8) {
        seed = seed % 10000;
        let ind = self.rarities.iter().position(|x| *x > seed).unwrap();
        (
            self.uris[ind][seed as usize % self.uris[ind].len()].clone(),
            ind as u8,
        )
    }
    pub fn default() -> Self {
        Self {
            validation_phrase: constants::URIS_ACCOUNT_VAL_PHRASE,
            rarities: Vec::new(),
            rarity_names: Vec::new(),
            uris: Vec::new(),
        }
    }
    pub fn get_space(&self) -> usize {
        let mut space = 4;
        space += self.rarities.len() * 2 + 4;
        self.rarity_names.iter().for_each(|x| {
            space += x.len() + 4;
        });
        space += 8;
        for i in self.uris.iter() {
            space += 4;
            for j in i.iter() {
                space += j.len() + 4;
            }
        }
        space
    }
}

#[derive(BorshDeserialize, Copy, Clone, PartialEq, Debug, BorshSerialize)]
/// This Stores the Cummulative of rewards for a specific vote account for the epoch the process_rewards instruction was run.
pub struct VoteReward {
    /// This is the epoch the reward was earned.
    pub epoch_number: u64,
    /// This is the amount of rewards earned.
    pub total_reward: u64,
    /// This is the total primary staked nft sol of the vote account before the process_rewards.
    pub total_stake: u64,
    /// This is the total reward that will be distributed to primary stakers.
    pub nft_holders_reward: u64,
}

impl VoteReward {
    pub fn get_space() -> usize {
        32
    }
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
impl RebalancingData {
    pub fn get_space() -> usize {
        17
    }
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
pub struct GeneralData {
    pub validation_phrase: u32,
    pub mint_numeration: u32,
    pub pending_delegation_total: u64,
    pub dealloced: u64,
    pub total_delegated: u64,
    pub last_withdraw_epoch: u64, //TODO: This isn't necessary right? What are the intentions behind this
    pub last_total_staked: u64,
    pub is_t_stake_initialized: bool,
    pub proposal_numeration: u32,
    pub last_feeless_redemption_date: u32,
    pub last_validated_validator_id_proposal: u32,
    pub rebalancing_data: RebalancingData,
    pub unfinalized_proposals: BTreeSet<u32>,
    pub vote_rewards: Vec<VoteReward>,
}
impl GeneralData {
    pub fn get_space(&self) -> usize {
        // 4 + 4 + 8 + 8 + 8 + 8 + 8 + 1 + 4 + 4 + 4 + RebalancingData::get_space() + (unfinalized_proposals.len() * 4 + 4) + (VoteReward::get_space() * self.vote_rewards.len() + 4)
        // 4 + 4 + 8 + 8 + 8 + 8 + 8 + 1 + 4 + 4 + 4 + 4 + 4 = 69
        69 + RebalancingData::get_space()
            + (VoteReward::get_space() * self.vote_rewards.len())
            + (self.unfinalized_proposals.len() * 4)
    }
}

impl Default for GeneralData {
    fn default() -> Self {
        Self {
            validation_phrase: constants::GENERAL_ACCOUNT_VAL_PHRASE,
            mint_numeration: 0,
            pending_delegation_total: 0,
            dealloced: 0,
            total_delegated: 0,
            last_withdraw_epoch: 0,
            last_total_staked: 0,
            is_t_stake_initialized: false,
            proposal_numeration: 0,
            last_feeless_redemption_date: 0,
            rebalancing_data: RebalancingData::default(),
            unfinalized_proposals: BTreeSet::new(),
            vote_rewards: Vec::new(),
            last_validated_validator_id_proposal: 0,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum FundsLocation {
    Delegated,
    Undelegated,
}

#[derive(BorshSerialize, BorshDeserialize, Validate)]
#[validation_phrase(crate::state::constants::NFT_DATA_VAL_PHRASE)]
//Creation Size:
pub struct NftData {
    pub validation_phrase: u32,
    pub rarity: Option<u8>,
    pub rarity_seed_time: Option<u32>,
    pub funds_location: FundsLocation,
    pub numeration: u32,
    pub date_created: u32,
    pub last_withdrawal_epoch: Option<u64>,
    pub last_delegation_epoch: Option<u64>,
    pub all_withdraws: Vec<u64>,
    pub all_votes: BTreeMap<u32, bool>,
}
impl NftData {
    pub fn get_space(&self) -> usize {
        // 4 + (1 + 1) + (1 + 4) + 1 + 4 + 4 + (1 + 8) + (1 + 8) + (8 * self.all_withdraws.len() + 4) + (5 * self.all_votes.len() + 4)
        // 4 + 1 + 1 + 1 + 4 + 1 + 4 + 4 + 9 + 9 + 4 + 4 = 40
        46 + (8 * self.all_withdraws.len()) + (5 * self.all_votes.len())
    }
}

pub fn get_feeds(network: &Network) -> Vec<Pubkey> {
    let mut feeds = Vec::new();
    match network {
        Network::Devnet => {
            for feed in constants::DEV_FEEDS {
                feeds.push(Pubkey::from_str(feed).unwrap());
            }
        }
        Network::Mainnet => {
            for feed in constants::MAIN_FEEDS {
                feeds.push(Pubkey::from_str(feed).unwrap());
            }
        }
        Network::LocalTest => {
            for feed in constants::LOCALTEST_FEEDS {
                feeds.push(Pubkey::from_str(feed).unwrap());
            }
        }
    }
    return feeds;
}

pub enum LogColors {
    Red,
    Green,
    Blue,
    Blank,
}

#[derive(BorshSerialize, Clone, BorshDeserialize)]
pub enum ConfigAccountType {
    MaxPrimaryStake(u64),
    NftHolderShare(u8),
    InitialRedemptionFee(u8),
    RedemptionFeeDuration(u32),
    ValidatorName(String),
    TwitterHandle(String),
    DiscordInvite(String),
}
impl ConfigAccountType {
    pub fn verify(&self) -> ProgramResult {
        match self {
            ConfigAccountType::MaxPrimaryStake(_) => (),
            ConfigAccountType::NftHolderShare(x) => {
                if *x > 100 {
                    Err(InglError::InvalidData.utilize("NFT Holder Share must be 100 or less"))?
                }
            }
            ConfigAccountType::InitialRedemptionFee(x) => {
                if *x > 100 {
                    Err(InglError::InvalidData
                        .utilize("Initial Redemption Fee must be 100 or less"))?
                }
            }
            ConfigAccountType::RedemptionFeeDuration(x) => {
                if *x > 86400 * 365 * 2 {
                    Err(InglError::InvalidData
                        .utilize("Early Redemption Fee cannot exceed 2 years"))?
                }
            }
            ConfigAccountType::ValidatorName(x) => {
                if x.len() > 32 {
                    Err(InglError::InvalidData
                        .utilize("Validator Name Can't be more than 32 characters"))?
                }
            }
            ConfigAccountType::TwitterHandle(x) => {
                if x.len() > 32 {
                    Err(InglError::InvalidData
                        .utilize("Twitter Handle Can't be more than 32 characters"))?
                }
            }
            ConfigAccountType::DiscordInvite(x) => {
                if x.len() > 32 {
                    Err(InglError::InvalidData
                        .utilize("Discord Invite Can't be more than 32 characters"))?
                }
            }
        };
        Ok(())
    }
}

#[derive(BorshSerialize, Clone, BorshDeserialize)]
pub enum VoteAccountGovernance {
    ValidatorID(Pubkey),
    Commission(u8),
}
impl VoteAccountGovernance {
    pub fn verify(&self) -> ProgramResult {
        match self {
            VoteAccountGovernance::ValidatorID(_) => (),
            VoteAccountGovernance::Commission(x) => {
                if *x > 100 {
                    Err(InglError::InvalidData.utilize("Commision Can't exceed 100"))?
                }
            }
        };
        Ok(())
    }
}

#[derive(BorshSerialize, Clone, BorshDeserialize)]
pub enum GovernanceType {
    ConfigAccount(ConfigAccountType),
    ProgramUpgrade {
        buffer_account: Pubkey,
        code_link: String,
    },
    VoteAccountGovernance(VoteAccountGovernance),
}
impl GovernanceType {
    pub fn verify(&self) -> ProgramResult {
        match self {
            GovernanceType::ConfigAccount(x) => x.verify(),
            GovernanceType::ProgramUpgrade { .. } => Ok(()),
            GovernanceType::VoteAccountGovernance(x) => x.verify(),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Validate)]
#[validation_phrase(crate::state::constants::GOVERNANCE_DATA_VAL_PHRASE)]
pub struct GovernanceData {
    pub validation_phrase: u32,
    pub expiration_time: u32,
    pub is_still_ongoing: bool,
    pub date_finalized: Option<u32>,
    pub did_proposal_pass: Option<bool>,
    pub is_proposal_executed: bool,
    pub title: String,
    pub description: String,
    pub votes: BTreeMap<u32, bool>,
    pub governance_type: GovernanceType,
}
impl GovernanceData {
    pub fn get_space(&self) -> usize {
        let mut space = 4 + 4 + 1 + 5 + 2 + 1;
        space += self.votes.len() * 5;
        space += 4 + self.title.len();
        space += 4 + self.description.len();

        space += 1 + match self.governance_type.clone() {
            GovernanceType::ConfigAccount(tmp) => match tmp {
                ConfigAccountType::MaxPrimaryStake(_) => 1 + 4,
                ConfigAccountType::NftHolderShare(_) => 1 + 1,
                ConfigAccountType::InitialRedemptionFee(_) => 1 + 1,
                ConfigAccountType::RedemptionFeeDuration(_) => 1 + 4,
                ConfigAccountType::ValidatorName(item) => 1 + 4 + item.len(),
                ConfigAccountType::TwitterHandle(item) => 1 + 4 + item.len(),
                ConfigAccountType::DiscordInvite(item) => 1 + 4 + item.len(),
            },
            GovernanceType::ProgramUpgrade {
                buffer_account: _,
                code_link,
            } => 32 + 4 + code_link.len(),

            GovernanceType::VoteAccountGovernance(tmp) => match tmp {
                VoteAccountGovernance::ValidatorID(_) => 1 + 32,
                VoteAccountGovernance::Commission(_) => 1 + 1,
            },
        };

        space
    }

    pub fn verify(&self) -> ProgramResult {
        if self.title.len() > 100 {
            Err(InglError::InvalidData.utilize("Title can't be more than 150 characters"))?
        }
        if self.description.len() > 350 {
            Err(InglError::InvalidData.utilize("Description can't be more than 1000 characters"))?
        }
        self.governance_type.verify()
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct VoteInit {
    pub node_pubkey: Pubkey,
    pub authorized_voter: Pubkey,
    pub authorized_withdrawer: Pubkey,
    pub commission: u8,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum VoteAuthorize {
    Voter,
    Withdrawer,
}

#[derive(Deserialize)]
pub struct VoteState {
    /// the node that votes in this account
    pub node_pubkey: Pubkey,

    /// the signer for withdrawals
    pub authorized_withdrawer: Pubkey,
    /// percentage (0-100) that represents what part of a rewards
    ///  payout should be given to this VoteAccount
    pub commission: u8,

    pub votes: VecDeque<Lockout>,

    // This usually the last Lockout which was popped from self.votes.
    // However, it can be arbitrary slot, when being used inside Tower
    pub root_slot: Option<Slot>,

    /// the signer for vote transactions
    pub authorized_voters: AuthorizedVoters,
    // OTHER FIELDS OMITTED INORDER TO DESERIALIZE ON THE STACK.
}
impl VoteState {
    pub fn space() -> usize {
        3731
    }
    pub fn min_lamports() -> u64 {
        Rent::get().unwrap().minimum_balance(Self::space())
    }
    pub fn deserialize(input: &[u8]) -> Box<Self> {
        let collected = deserialize::<VoteStateVersions>(input).unwrap();
        Box::new(collected.convert_to_current())
    }
}

pub type UnixTimestamp = i64;
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct BlockTimestamp {
    pub slot: Slot,
    pub timestamp: UnixTimestamp,
}
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AuthorizedVoters {
    pub authorized_voters: BTreeMap<Epoch, Pubkey>,
}
impl AuthorizedVoters {
    pub fn new(epoch: Epoch, pubkey: Pubkey) -> Self {
        let mut authorized_voters = BTreeMap::new();
        authorized_voters.insert(epoch, pubkey);
        Self { authorized_voters }
    }
    pub fn last(&self) -> Option<(&u64, &Pubkey)> {
        self.authorized_voters.iter().next_back()
    }
}

#[derive(Serialize, Default, Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub struct Lockout {
    pub slot: Slot,
    pub confirmation_count: u32,
}

#[derive(Deserialize)]
pub enum VoteStateVersions {
    V0_23_5(Box<VoteState0_23_5>),
    Current(Box<VoteState>),
}

impl VoteStateVersions {
    pub fn convert_to_current(self) -> VoteState {
        match self {
            VoteStateVersions::V0_23_5(state) => {
                let authorized_voters =
                    AuthorizedVoters::new(state.authorized_voter_epoch, state.authorized_voter);

                VoteState {
                    node_pubkey: state.node_pubkey,

                    /// the signer for withdrawals
                    authorized_withdrawer: state.authorized_withdrawer,

                    /// percentage (0-100) that represents what part of a rewards
                    ///  payout should be given to this VoteAccount
                    commission: state.commission,

                    votes: VecDeque::new(),

                    root_slot: None,

                    /// the signer for vote transactions
                    authorized_voters,
                }
            }
            VoteStateVersions::Current(state) => *state,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct VoteState0_23_5 {
    /// the node that votes in this account
    pub node_pubkey: Pubkey,

    /// the signer for vote transactions
    pub authorized_voter: Pubkey,
    /// when the authorized voter was set/initialized
    pub authorized_voter_epoch: Epoch,

    /// history of prior authorized voters and the epoch ranges for which
    ///  they were set
    pub prior_voters: CircBuf<(Pubkey, Epoch, Epoch, Slot)>,

    /// the signer for withdrawals
    pub authorized_withdrawer: Pubkey,
    /// percentage (0-100) that represents what part of a rewards
    ///  payout should be given to this VoteAccount
    pub commission: u8,
    // OTHER FIELDS OMITTED INORDER TO DESERIALIZE ON THE STACK.
}

const MAX_ITEMS: usize = 32;
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct CircBuf<I> {
    pub buf: [I; MAX_ITEMS],
    /// next pointer
    pub idx: usize,
}
impl<I: Default + Copy> Default for CircBuf<I> {
    fn default() -> Self {
        Self {
            buf: [I::default(); MAX_ITEMS],
            idx: MAX_ITEMS - 1,
        }
    }
}

impl<I> CircBuf<I> {
    pub fn append(&mut self, item: I) {
        // remember prior delegate and when we switched, to support later slashing
        self.idx += 1;
        self.idx %= MAX_ITEMS;

        self.buf[self.idx] = item;
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum UpgradeableLoaderState {
    /// Account is not initialized.
    Uninitialized,
    /// A Buffer account.
    Buffer {
        /// Authority address
        authority_address: Option<Pubkey>,
        // The raw program data follows this serialized structure in the
        // account's data.
    },
    /// An Program account.
    Program {
        /// Address of the ProgramData account.
        programdata_address: Pubkey,
    },
    // A ProgramData account.
    ProgramData {
        /// Slot that the program was last modified.
        slot: u64,
        /// Address of the Program's upgrade authority.
        upgrade_authority_address: Option<Pubkey>, // TODO: Check that the upgrade_authority_address is a signer during intialization.
                                                   // The raw program data follows this serialized structure in the
                                                   // account's data.
    },
}
