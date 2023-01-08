#![allow(unused_parens)]
use crate::{colored_log, error::InglError, utils::{assert_program_owned, ResultExt, AccountInfoHelpers}};
use borsh::{BorshDeserialize, BorshSerialize};
use ingl_macros::Validate;
use serde::{Deserialize, Serialize};
use solana_program::{
    account_info::AccountInfo,
    borsh::try_from_slice_unchecked,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use crate::state::LogColors::{*};
pub const LOG_LEVEL: u8 = 5;

pub mod constants {
    pub const INGL_CONFIG_VAL_PHRASE: u32 = 739_215_648;
    pub const URIS_ACCOUNT_VAL_PHRASE: u32 = 382_916_043;

    pub mod initializer{
        solana_program::declare_id!("62uPowNXr22WPw7XghajJkWMBJ2fnv1oGthxqHYYPHie");
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
    pub validator_name: String,
    pub twitter_handle: String,
    pub discord_invite: String,
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
