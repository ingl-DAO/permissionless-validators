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
        init_processes::{init::process_init, reset_uris::reset_uris, upload_uris::upload_uris},
        nft_processes::{imprint_rarity::process_imprint_rarity, mint_nft::process_mint_nft},
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
            proposal_quorum: program_upgrade_threshold,
            creator_royalties,
            governance_expiration_time,
            rarities,
            rarity_names,
            twitter_handle,
            discord_invite,
            validator_name,
            collection_uri,
            website,
            default_uri,
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
            governance_expiration_time,
            twitter_handle,
            discord_invite,
            validator_name,
            collection_uri,
            website,
            default_uri,
        )?,
        InstructionEnum::CreateVoteAccount { log_level } => {
            create_vote_account(program_id, accounts, log_level, false)?
        }

        InstructionEnum::MintNft {
            log_level,
        } => process_mint_nft(
            program_id,
            accounts,
            log_level,
            false,
        )?,
        InstructionEnum::ImprintRarity { log_level } => {
            process_imprint_rarity(program_id, accounts, log_level, false)?
        }

        InstructionEnum::UploadUris {
            uris,
            rarity,
            log_level,
        } => upload_uris(program_id, accounts, uris, rarity, log_level)?,

        InstructionEnum::InitGovernance {
            log_level,
            governance_type,
            title,
            description,
        } => create_governance(
            program_id,
            accounts,
            governance_type,
            title,
            description,
            log_level,
            false,
            false,
        )?,

        InstructionEnum::VoteGovernance {
            log_level,
            numeration,
            vote,
            cnt,
        } => vote_governance(
            program_id, accounts, numeration, vote, cnt, log_level, false, false,
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

        InstructionEnum::ResetUris { log_level } => {
            reset_uris(program_id, accounts, log_level)?;
        }

        _ => {
            log!(0, 5, "Instruction not yet Implemented");
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    Ok(())
}
