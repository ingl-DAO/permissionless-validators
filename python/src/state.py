import json
from typing import Optional
import base58
from borsh_construct import *
from .ledger import *
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.rpc import types
from solana.rpc.async_api import AsyncClient
from solana.transaction import Transaction
from solders.rpc.responses import SendTransactionResp
import os


class Constants:
    INGL_CONFIG_SEED = b"ingl_config";
    URIS_ACCOUNT_SEED = b"uris_account";
    GENERAL_ACCOUNT_SEED = b"general_account";
    INGL_NFT_COLLECTION_KEY = b"ingl_nft_collection";
    INGL_MINT_AUTHORITY_KEY = b"ingl_mint_authority";
    COLLECTION_HOLDER_KEY = b"collection_holder";
    VOTE_ACCOUNT_KEY = b"vote_account";
    AUTHORIZED_WITHDRAWER_KEY = b"authorized_withdrawer";
    STAKE_ACCOUNT_KEY = b"stake_account";
    PD_POOL_ACCOUNT_KEY = b"pd_pool_account";
    NFT_ACCOUNT_CONST = b"nft_account";
    INGL_PROGRAM_AUTHORITY_KEY = b"ingl_program_authority";
    INGL_PROPOSAL_KEY = b"ingl_proposal";
    VALIDATOR_ID_SEED = b"validator_ID___________________";
    T_STAKE_ACCOUNT_KEY = b"t_stake_account_key";
    T_WITHDRAW_KEY = b"t_withdraw_key";

    TEAM_ACCOUNT_KEY = PublicKey("Team111111111111111111111111111111111111111")

ClassEnum = Enum(
    "Ruby",
    "Diamond",
    "Sapphire",
    "Emerald",
    "Serendibite",
    "Benitoite",

    enum_name = "ClassEnum",
)

Rarity = Enum(
    "Common",
    "Uncommon",
    "Rare",
    "Exalted",
    "Mythic",

    enum_name = "Rarity",
)

def int_from_class_enum(class_enum: ClassEnum.enum) -> int:
    if class_enum == ClassEnum.enum.Ruby():
        return 0
    elif class_enum == ClassEnum.enum.Diamond():
        return 1
    elif class_enum == ClassEnum.enum.Sapphire():
        return 2
    elif class_enum == ClassEnum.enum.Emerald():
        return 3
    elif class_enum == ClassEnum.enum.Serendibite():
        return 4
    elif class_enum == ClassEnum.enum.Benitoite():
        return 5

def keypair_from_json(filepath):
    keypair = Keypair.from_secret_key(json.load(open(filepath)))
    return keypair

def pubkey_from_json(filepath): #Not Tested yet.
    return PublicKey(json.load(filepath.open()))

GlobalGems = CStruct(
    "validation_phrase"/ U32,
    "counter" / U32,
    "total_raised" / U64,
    "pd_pool_total" / U64,
    "delegated_total" / U64,
    "dealloced_total" / U64,
    "is_proposal_ongoing" / Bool,
    "proposal_numeration" / U32,
    "pending_delegation_total" / U64,
    "upgrade_proposal_numeration" / U32,
    "validator_list" / Vec(U8[32])
)

VoteRewards = CStruct(
    "validation_phrase" / U32,
    "epoch_number" / U64,
    "total_reward" / U64,
    "total_stake" / U64,
)

InglVoteAccountData = CStruct(
    "validation_phrase" / U32,
    "total_delegated" / U64,
    "last_withdraw_epoch" / U64,
    "dealloced" / U64,
    "rebalancing_data" / CStruct(
        "pending_validator_rewards" / U64,
        "unclaimed_validator_rewards" / U64,
        "is_rebalancing_active" / Bool
    ), # Field is also used to check if there is an ongoing rebalancing or not.
    "validator_id" / U8[32], #To Reconsider.
    "last_total_staked" / U64,
    "is_t_stake_initialized" / Bool,
    "pending_delegation_total" / U64,
    "vote_rewards" / Vec(VoteRewards),
)

ValidatorProposal = CStruct(
    "validation_phrase" / U32,
    "validator_ids" / Vec(U8[32]),
    "date_created" / U32,
    "date_finalized" / Option(U32),
    "votes" / Vec(U32),
    "winner" / Option(U8[32]),
)

UpgradeVote = CStruct(
    "vote" / Bool,
    "validator_id" / U8[32],
)

ProgramUpgradeData = CStruct(
    "validation_phrase" / U32,
    "buffer_address" / U8[32],
    "code_link" / String,
    "is_still_ongoing" / Bool,
    "votes" / HashMap(U8[32], UpgradeVote),
)

ValidatorConfig = CStruct(
    "validation_phrase" / U32,
    "is_validator_id_switchable" / Bool,
    "max_primary_stake" / U64,
    "nft_holders_share" / U8,
    "initial_redemption_fee" / U8,
    "unit_stake" / U64,
    "redemption_fee_duration" / U32,
    "proposal_quorum" / U8,
    "creator_royalties" / U16,
    "commission" / U8,
    "validator_id" / U8[32],
    "validator_name" / String,
    "twitter_handle" / String,
    "discord_invite" / String,
    "website" / String,
)


def private_key_from_json(filepath):
    return base58.b58encode(keypair_from_json(filepath).secret_key).decode()

class rpc_url:
    DEVNET = "https://api.devnet.solana.com"
    TESTNET = "https://api.testnet.solana.com"
    MAINNET = "https://api.mainnet.solana.com"
    target_network = DEVNET
    
    def get_explorer_suffix():
        if rpc_url.target_network == rpc_url.DEVNET:
            return "?cluster=devnet"
        elif rpc_url.target_network == rpc_url.TESTNET:
            return "?cluster=testnet"
        else:
            return ""

    def get_network_url(network):
        if network == "devnet":
            return rpc_url.DEVNET
        elif network == "testnet":
            return rpc_url.TESTNET
        elif network == "mainnet":
            return rpc_url.MAINNET
        else:
            raise Exception("Invalid network")



class KeypairInput:
    def __init__(self, keypair: Optional[Keypair] = None, ledger_address: Optional[int] = None, pubkey: Optional[PublicKey] = None):
        assert keypair or ledger_address, "KeypairInput must have at least one of keypair or ledger_address"
        self.keypair = keypair
        self.ledger_address = ledger_address
        self.public_key = pubkey if pubkey else keypair.public_key if keypair else None

def parse_keypair_input(str_input: String) -> KeypairInput:
    if str_input.startswith("Ledger://"):
        t_dongle = ledgerDongle()
        pub_key = t_dongle.get_address(int(str_input[9:]))
        return KeypairInput(ledger_address=int(str_input[9:]), pubkey=pub_key)
    else:
        t_keypair=keypair_from_json(str_input)
        return KeypairInput(keypair=t_keypair, pubkey=t_keypair.public_key)
class PubkeyInput:
    def __init__(self, keypair: Optional[Keypair] = None, pubkey: Optional[PublicKey] = None, ledger_address: Optional[int] = None):
        assert keypair or pubkey or ledger_address, "PubkeyInput must have at least one of keypair, pubkey or ledger_address"
        self.keypair = keypair
        self.public_key = keypair.public_key if keypair else pubkey
        self.ledger_address = ledger_address

def parse_pubkey_input(str_input: String) -> PubkeyInput:
    if str_input.startswith("Ledger://"):
        t_dongle = ledgerDongle()
        pub_key = t_dongle.get_address(int(str_input[9:]))
        return PubkeyInput(ledger_address=int(str_input[9:]), pubkey=pub_key)
    else:
        try:
            pubkey = PubkeyInput(pubkey=PublicKey(str_input))
            return pubkey
        except Exception as e:
            if 'invalid public key input:' in str(e):
                try:
                    t_keypair = keypair_from_json(str_input)
                    return PubkeyInput(keypair=t_keypair, pubkey=t_keypair.public_key)
                except Exception as new_e:
                    print("invalid public key input")
                    raise new_e
            else:
                print("invalid public key input")
                raise e

async def sign_and_send_tx(tx: Transaction, client: AsyncClient, *args) -> SendTransactionResp:
    last_valid_block_height = None
    if client.blockhash_cache:
        try:
            recent_blockhash = client.blockhash_cache.get()
        except ValueError:
            blockhash_resp = await client.get_latest_blockhash(Finalized)
            recent_blockhash = client._process_blockhash_resp(blockhash_resp, used_immediately=True)
            last_valid_block_height = blockhash_resp.value.last_valid_block_height
    else:
        blockhash_resp = await client.get_latest_blockhash(Finalized)
        recent_blockhash = client.parse_recent_blockhash(blockhash_resp)
        last_valid_block_height = blockhash_resp.value.last_valid_block_height
    tx.recent_blockhash = recent_blockhash

    # print("signing actually args: ", args)
    for arg in args:
        # print(arg)
        if isinstance(arg, KeypairInput):
            # print("p_key: ", arg.public_key, "keypair: ", arg.keypair, "ledger: ", arg.ledger_address)
            if arg.keypair is not None:
                tx.sign_partial(arg.keypair)
            elif arg.ledger_address is not None:
                t_dongle = ledgerDongle()
                message = await make_message(tx, client, False)
                # print("message: ", message)
                signature = Signature.from_bytes(t_dongle.sign(message, arg.ledger_address))
                tx.add_signature(arg.public_key, signature)
            else:
                raise Exception("KeypairInput is not valid")
        else:
            raise ValueError("Invalid argument expected a KeypairInput, Found -> : " + str(type(arg)))
    # print("Reached here")
    opts_to_use = types.TxOpts(preflight_commitment=client._commitment, last_valid_block_height=last_valid_block_height)
    txn_resp = await client.send_raw_transaction(tx.serialize(), opts=opts_to_use)
    if client.blockhash_cache:
        blockhash_resp = await client.get_latest_blockhash(Finalized)
        client._process_blockhash_resp(blockhash_resp, used_immediately=False)
    # print("finished")
    return txn_resp


def parse_upgrade_proposal_id(proposal_pubkey: Optional[PublicKey], numeration: Optional[int], cnt: int) -> Tuple[PublicKey, int]:
    proposal_account_pubkey = PublicKey(1)
    proposal_numeration = 0
    if numeration:
        proposal_account_pubkey = PublicKey.find_program_address([bytes(Constants.UPGRADE_PROPOSAL_KEY, 'UTF-8'), (numeration).to_bytes(4,"big")], get_program_id())[0]
        proposal_numeration = numeration
        return proposal_account_pubkey, proposal_numeration
    else:
       while cnt > 0:
            proposal_account_pubkey = PublicKey.find_program_address([bytes(Constants.UPGRADE_PROPOSAL_KEY, 'UTF-8'), (cnt-1).to_bytes(4,"big")], get_program_id())[0]
            if proposal_account_pubkey == proposal_pubkey.public_key:
                proposal_numeration = cnt
                break
            else:
                cnt -=1
    if proposal_account_pubkey != proposal_pubkey.public_key:
        raise Exception("Proposal not found")
    return proposal_account_pubkey, proposal_numeration

def parse_validator_proposal_id(proposal_pubkey: Optional[PublicKey], numeration: Optional[int], cnt: int) -> Tuple[PublicKey, int]:
    proposal_account_pubkey = PublicKey(1)
    proposal_numeration = 0
    if numeration:
        proposal_account_pubkey = PublicKey.find_program_address([bytes(Constants.VOTE_ACCOUNT_KEY, 'UTF-8'), (numeration).to_bytes(4,"big")], get_program_id())[0]
        proposal_numeration = numeration
        return proposal_account_pubkey, proposal_numeration
    else:
       while cnt > 0:
            proposal_account_pubkey = PublicKey.find_program_address([bytes(Constants.VOTE_ACCOUNT_KEY, 'UTF-8'), (cnt-1).to_bytes(4,"big")], get_program_id())[0]
            if proposal_account_pubkey == proposal_pubkey.public_key:
                proposal_numeration = cnt
                break
            else:
                cnt -=1
    if proposal_account_pubkey != proposal_pubkey.public_key:
        raise Exception("Proposal not found")
    return proposal_account_pubkey, proposal_numeration


def set_config(key: str, value: str):
    file_dir = f"{os.path.expanduser('~')}/.config/solana/ingl/"
    os.makedirs(file_dir, exist_ok=True)
    file_dir = file_dir + "config.json"
    try:
        f = open(file_dir, 'r')
        config = json.load(f)
        f.close()
    except:
        config = {}
    config[key] = value
    with open(file_dir, 'w') as f:
        json.dump(config, f)

def get_config(key: str) -> str:
    file_dir = f"{os.path.expanduser('~')}/.config/solana/ingl/"
    os.makedirs(file_dir, exist_ok=True)
    file_dir = file_dir + "config.json"
    try:
        f = open(file_dir, 'r')
        config = json.load(f)
        f.close()
        if key in config:
            return config[key]
        else:
            return ""
    except:
        return ""


def get_program_id() -> PublicKey:
    program_id_str = get_config('program_id')
    try:
        return PublicKey(program_id_str)
    except:
        return PublicKey("HD8kYhgqmZCJ881vyBQ3fR6a62YL7cZBnYj1P7oLw8An")

def set_program_id(program_id: str):
    set_config('program_id', program_id)

def get_network() -> str:
    network = get_config('network')
    if network == "":
        return rpc_url.get_network_url('devnet')
    else:
        return network

def set_network(network: str):
    set_config('network', network)

def get_keypair_path() -> str:
    keypair_path = get_config('keypair_path')
    if keypair_path == "":
        return f"{os.path.expanduser('~')}/.config/solana/ingl/id.json"
    else:
        return keypair_path
    
def set_keypair_path(keypair_path: str):
    set_config('keypair_path', keypair_path)