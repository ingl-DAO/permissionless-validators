use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    instruction::InstructionEnum,
    log,
    processes::{
        governance_processes::{
            execute_governance::execute_governance, finalize_governance::finalize_governance,
            init_governance::create_governance, vote_governance::vote_governance,
        },
        init_processes::{init::process_init, upload_uris::upload_uris},
        nft_processes::mint_nft::process_mint_nft,
        rewards_processes::{
            finalize_rebalance::finalize_rebalance, init_rebalance::init_rebalance,
            nft_withdraw::nft_withdraw, process_rewards::process_rewards,
        },
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
            init_commission,
            max_primary_stake,
            nft_holders_share,
            initial_redemption_fee,
            is_validator_id_switchable,
            unit_backing,
            redemption_fee_duration,
            program_upgrade_threshold,
            creator_royalties,
            rarities,
            rarity_names,
            twitter_handle,
            discord_invite,
            validator_name,
            collection_uri,
            website,
        } => process_init(
            program_id,
            accounts,
            log_level,
            init_commission,
            max_primary_stake,
            nft_holders_share,
            initial_redemption_fee,
            is_validator_id_switchable,
            unit_backing,
            redemption_fee_duration,
            program_upgrade_threshold,
            creator_royalties,
            rarities,
            rarity_names,
            twitter_handle,
            discord_invite,
            validator_name,
            collection_uri,
            website,
        )?,
        InstructionEnum::CreateVoteAccount { log_level } => {
            create_vote_account(program_id, accounts, log_level, false)?
        }

        InstructionEnum::MintNft {
            log_level,
            switchboard_state_bump,
            permission_bump,
        } => process_mint_nft(
            program_id,
            accounts,
            switchboard_state_bump,
            permission_bump,
            log_level,
            false,
        )?,

        InstructionEnum::UploadUris {
            uris,
            rarity,
            log_level,
        } => upload_uris(program_id, accounts, uris, rarity, log_level)?,

        InstructionEnum::InitGovernance {
            log_level,
            governance_type,
        } => create_governance(
            program_id,
            accounts,
            governance_type,
            log_level,
            false,
            false,
        )?,

        InstructionEnum::VoteGovernance {
            log_level,
            numeration,
            vote,
        } => vote_governance(
            program_id, accounts, numeration, vote, log_level, false, false,
        )?,

        InstructionEnum::FinalizeGovernance {
            numeration,
            log_level,
        } => finalize_governance(program_id, accounts, numeration, log_level)?,

        InstructionEnum::ExecuteGovernance {
            numeration,
            log_level,
        } => execute_governance(program_id, accounts, numeration, log_level)?,

        InstructionEnum::NFTWithdraw { cnt, log_level } => {
            nft_withdraw(program_id, accounts, cnt, log_level, false, false)?
        }

        InstructionEnum::ProcessRewards { log_level } => {
            process_rewards(program_id, accounts, log_level, false, false)?
        }

        InstructionEnum::InitRebalance { log_level } => {
            init_rebalance(program_id, accounts, log_level)?
        }

        InstructionEnum::FinalizeRebalance { log_level } => {
            finalize_rebalance(program_id, accounts, log_level)?
        }

        _ => {
            log!(0, 5, "Instruction not yet Implemented");
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    Ok(())
}
