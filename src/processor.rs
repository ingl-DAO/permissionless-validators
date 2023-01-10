use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    instruction::InstructionEnum,
    log,
    processes::{
        init_processes::init::process_init, nft_processes::mint_nft::process_mint_nft,
        validator_processes::create_vote_account::create_vote_account,
    },
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    match InstructionEnum::decode(data) {
        InstructionEnum::Init {
            log_level,
            init_commision,
            max_primary_stake,
            nft_holders_share,
            initial_redemption_fee,
            is_validator_id_switchable,
            unit_backing,
            redemption_fee_duration,
            program_upgrade_threshold,
            creator_royalties,
            rarities,
            twitter_handle,
            discord_invite,
            validator_name,
            collection_uri,
        } => process_init(
            program_id,
            accounts,
            log_level,
            init_commision,
            max_primary_stake,
            nft_holders_share,
            initial_redemption_fee,
            is_validator_id_switchable,
            unit_backing,
            redemption_fee_duration,
            program_upgrade_threshold,
            creator_royalties,
            rarities,
            twitter_handle,
            discord_invite,
            validator_name,
            collection_uri,
        )?,
        InstructionEnum::CreateVoteAccount { log_level } => {
            create_vote_account(program_id, accounts, log_level, false)?
        }

        InstructionEnum::MintNft { log_level } => {
            process_mint_nft(program_id, accounts, log_level, false)?
        }

        _ => {
            log!(0, 5, "Instruction not yet Implemented");
        }
    }

    Ok(())
}
