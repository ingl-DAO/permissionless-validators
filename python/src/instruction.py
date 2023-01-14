from typing import Optional
from borsh_construct import *
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
from .state import ClassEnum, Rarity


InitStruct = CStruct(
    "log_level" / U8,
    "init_commission" / U8,
    "max_primary_stake" / U64,
    "nft_holder_share" / U8,
    "initial_redemption_fee" / U8,
    "is_validator_id_switchable" / Bool,
    "unit_backing" / U64,
    "redemption_fee_duration" / U32,
    "program_upgrade_threshold" / U8,
    "creator_royalties" / U16,
    "rarities" / Vec(U16),
    "rarity_names" / Vec(String),
    "twitter_handle" / String,
    "discord_invite" / String,
    "validator_name" / String,
    "collection_uri" / String,
    "website" / String,
)


InstructionEnum = Enum(
    "MintNft" / CStruct("log_level"/U8),
    "Init" / InitStruct,
    "Redeem" / CStruct("log_level"/U8),
    "NFTWithdraw" / CStruct("cnt" / U32, "log_level"/U8),
    "ProcessRewards" / CStruct("log_level"/U8),
    "InitRebalance" / CStruct("log_level"/U8),
    "FinalizeRebalance" / CStruct("log_level"/U8),
    "UploadUris" / CStruct("uris"/ Vec(String), "rarity"/U8, "log_level"/U8),
    "ResetUris" / CStruct("log_level"/U8),
    "UnDelegateNFT" / CStruct("log_level"/U8),
    "DelegateNFT" / CStruct("log_level"/U8),
    "CreateVoteAccount" / CStruct("log_level"/U8),
    "InitGovernance",
    "VoteGovernance" / CStruct("numeration" / U32, "vote"/Bool, "cnt"/U8, "log_level"/U8),
    "FinalizeGovernance" / CStruct("numeration"/U32, "log_level"/U8),
    "ExecuteGovernance" / CStruct("numeration"/U32, "log_level"/U8),
    
    enum_name = "InstructionEnum",
)

GovernanceType = Enum(
    "ConfigAccount",
    "ProgramUpgrade" / CStruct("buffer_account" / U8[32], "code_link" / String),
    "VoteAccountGovernance",
)

ConfigAccountType = Enum(
    "MaxPrimaryStake" / U64,
    "NftHolderShare" / U8,
    "InitialRedemptionFee" / U8,
    "RedemptionFeeDuration" / U32,
    "ValidatorName" / String,
    "TwitterHandle" / String,
    "DiscordInvite" / String,
)

VoteAccountGovernance = Enum(
    "ValidatorId" / U8[32],
    "Commission" / U8,
)

def build_governance_type(governance_type: GovernanceType.enum, config_account_type:Optional[ConfigAccountType.enum] = None, vote_account_governance: Optional[VoteAccountGovernance.enum] = None):
    if governance_type != GovernanceType.enum.ProgramUpgrade():
        if governance_type == GovernanceType.enum.ConfigAccount():
            return GovernanceType.build(governance_type) + ConfigAccountType.build(config_account_type)
        elif governance_type == GovernanceType.enum.VoteAccountGovernance():
            return GovernanceType.build(governance_type) + VoteAccountGovernance.build(vote_account_governance)
        else:
            raise Exception("Invalid governance type")
    else:
        return GovernanceType.build(governance_type)

def build_instruction(instruction: InstructionEnum.enum, governance_type: Optional[GovernanceType.enum] = None, config_account_type:Optional[ConfigAccountType.enum] = None, vote_account_governance: Optional[VoteAccountGovernance.enum] = None, log_level: Optional[int] = None):
    if instruction == InstructionEnum.enum.InitGovernance():
        return InstructionEnum.build(instruction) +  build_governance_type(governance_type, config_account_type=config_account_type, vote_account_governance=vote_account_governance) + (log_level).to_bytes(1, "big")
    else:
        return InstructionEnum.build(instruction)


class ComputeBudgetInstruction:
    def __init__(self):
        self.InstructionEnum = Enum(
            "RequestUnitsDeprecated" / CStruct("units" / U32, "additional_fee"/U32),
            "RequestHeapFrame"/ CStruct("value" / U32),
            "SetComputeUnitLimit" / CStruct("value" / U32),
            "SetComputeUnitPrice" / CStruct("value" / U64),

            enum_name = 'InstructionEnum',
        )
        self.program_id = PublicKey("ComputeBudget111111111111111111111111111111")

    def request_heap_frame(self, total_bytes, payer) -> TransactionInstruction:
        instruction_bytes = self.InstructionEnum.build(self.InstructionEnum.enum.RequestHeapFrame(total_bytes))
        return TransactionInstruction(keys = [AccountMeta(payer, True, False)], program_id=self.program_id, data=instruction_bytes)

    def set_compute_unit_limit(self, units, payer) -> TransactionInstruction:
        instruction_bytes = self.InstructionEnum.build(self.InstructionEnum.enum.SetComputeUnitLimit(units))
        return TransactionInstruction(keys = [AccountMeta(payer, True, False)], program_id=self.program_id, data=instruction_bytes)

    def set_compute_unit_price(self, micro_lamports, payer) -> TransactionInstruction:
        instruction_bytes = self.InstructionEnum.build(self.InstructionEnum.enum.SetComputeUnitPrice(micro_lamports))
        return TransactionInstruction(keys = [AccountMeta(payer, True, False)], program_id=self.program_id, data=instruction_bytes)