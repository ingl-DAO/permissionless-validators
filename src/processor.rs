use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    instruction::{InstructionEnum},
    processes::{
        governance_processes::{
            execute_governance::execute_governance, finalize_governance::finalize_governance,
            init_governance::create_governance, vote_governance::vote_governance,
        },
        init_processes::{init::process_init, reset_uris::reset_uris, upload_uris::upload_uris, fractionalize_existing::fractionalize},
        nft_processes::{
            delegate_nft::delegate_gem, imprint_rarity::process_imprint_rarity,
            mint_nft::process_mint_nft, redeem_nft::redeem_nft, undelegate_nft::undelegate_nft,
        },
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
        InstructionEnum::Init(init_args) => process_init(
            program_id,
            accounts,
            init_args
        )?,
        InstructionEnum::CreateVoteAccount { log_level } => {
            create_vote_account(program_id, accounts, log_level, false)?
        }

        InstructionEnum::MintNft { log_level } => {
            process_mint_nft(program_id, accounts, log_level, false)?
        }
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

        InstructionEnum::UnDelegateNFT { log_level } => {
            undelegate_nft(program_id, accounts, log_level, false, false)?;
        }

        InstructionEnum::DelegateNFT { log_level } => {
            delegate_gem(program_id, accounts, log_level, false)?;
        }
        InstructionEnum::Redeem { log_level } => {
            redeem_nft(program_id, accounts, log_level, false)?
        }

        InstructionEnum::InjectTestingData {
            num_mints,
            log_level,
        } => injects::inject_testing_data(program_id, accounts, num_mints, log_level)?,

        InstructionEnum::FractionalizeExisting(init_args) => fractionalize(program_id, accounts, init_args)?,
    }

    Ok(())
}

pub mod injects {
    use solana_program::{
        account_info::next_account_info,
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        msg,
        native_token::LAMPORTS_PER_SOL,
        program::invoke,
        pubkey::Pubkey,
        system_instruction,
        sysvar::{clock::Clock, rent::Rent, Sysvar},
    };

    use borsh::BorshSerialize;

    use crate::{
        log,
        state::{
            constants::{GENERAL_ACCOUNT_SEED, NFT_ACCOUNT_CONST},
            FundsLocation, GeneralData, NftData, VoteReward,
        },
        utils::{AccountInfoHelpers, ResultExt},
    };
    pub fn inject_testing_data(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        num_mints: u8,
        log_level: u8,
    ) -> ProgramResult {
        //Remember to get rid of this function after accumulating enough testing sols to launch a 10,000Sol validator.
        let account_info_iter = &mut accounts.iter();
        let payer_account_info = next_account_info(account_info_iter)?;
        let general_data_info = next_account_info(account_info_iter)?;
        let authorized_withdrawer_info = next_account_info(account_info_iter)?;

        let (_expected_vote_data_pubkey, _expected_vote_data_bump) =
            general_data_info.assert_seed(program_id, &[GENERAL_ACCOUNT_SEED.as_ref()])?;
        general_data_info.assert_owner(program_id)?;
        let mut general_data = Box::new(GeneralData::parse(general_data_info, program_id)?);

        let chosen_epoch = Clock::get()?.epoch.saturating_sub(1);
        for _ in 0..num_mints {
            let mint_account_info = next_account_info(account_info_iter)?;
            let nft_account_data_info = next_account_info(account_info_iter)?;

            let (_gem_account_pubkey, _gem_account_bump) = nft_account_data_info.assert_seed(
                program_id,
                &[NFT_ACCOUNT_CONST.as_ref(), mint_account_info.key.as_ref()],
            )?;
            nft_account_data_info.assert_owner(program_id)?;
            mint_account_info.assert_owner(&spl_token::id())?;
            let mut nft_account_data = NftData::parse(nft_account_data_info, program_id)?;

            if let FundsLocation::Undelegated = nft_account_data.funds_location {
                msg!("Funds are already undelegated");
            }
            nft_account_data.last_delegation_epoch = Some(chosen_epoch - 4);
            nft_account_data.last_withdrawal_epoch = Some(chosen_epoch - 4);
            nft_account_data.serialize(&mut &mut nft_account_data_info.data.borrow_mut()[..])?;
        }
        log!(log_level, 2, "Transfering the tokens ...");
        invoke(
            &system_instruction::transfer(
                payer_account_info.key,
                authorized_withdrawer_info.key,
                LAMPORTS_PER_SOL.checked_mul(4).unwrap(),
            ),
            &[
                payer_account_info.clone(),
                authorized_withdrawer_info.clone(),
            ],
        )?;

        general_data.vote_rewards = Vec::new();
        general_data.vote_rewards.push(VoteReward {
            epoch_number: chosen_epoch - 2,
            total_stake: general_data.total_delegated,
            nft_holders_reward: LAMPORTS_PER_SOL - (0.1 * LAMPORTS_PER_SOL as f64) as u64,
            total_reward: 1 * LAMPORTS_PER_SOL,
        });
        general_data.vote_rewards.push(VoteReward {
            epoch_number: chosen_epoch - 1,
            total_stake: general_data.total_delegated,
            nft_holders_reward: LAMPORTS_PER_SOL - (0.1 * LAMPORTS_PER_SOL as f64) as u64,
            total_reward: 1 * LAMPORTS_PER_SOL,
        });
        general_data.vote_rewards.push(VoteReward {
            epoch_number: chosen_epoch,
            total_stake: general_data.total_delegated,
            nft_holders_reward: 2 * (LAMPORTS_PER_SOL - (0.1 * LAMPORTS_PER_SOL as f64) as u64),
            total_reward: 2 * LAMPORTS_PER_SOL,
        });
        general_data.last_withdraw_epoch = chosen_epoch - 1;

        let new_space = general_data.get_space();
        let lamports = Rent::get()?
            .minimum_balance(new_space)
            .checked_sub(Rent::get()?.minimum_balance(general_data_info.data.borrow().len()))
            .unwrap();

        invoke(
            &system_instruction::transfer(payer_account_info.key, general_data_info.key, lamports),
            &[payer_account_info.clone(), general_data_info.clone()],
        )
        .error_log(
            "failed to transfer for reallaocating general data account size @system_program invoke",
        )?;

        general_data_info
            .realloc(new_space, false)
            .error_log("failed to realloc general data account size")?;

        general_data.serialize(&mut &mut general_data_info.data.borrow_mut()[..])?;

        Ok(())
    }
}
