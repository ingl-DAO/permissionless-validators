import asyncclick as click
from .instruction import *
from .processor import *
from solana.keypair import Keypair
from solana.rpc.api import Client as Uasyncclient
from solana.publickey import PublicKey
from borsh_construct import *
from .state import *
from .utils import *
from .state import Constants as ingl_constants
from rich import print
from solana.rpc.async_api import AsyncClient
from .cli_state import CLI_VERSION
import os
uasyncclient = Uasyncclient(rpc_url.target_network)

@click.group()
@click.version_option(version=CLI_VERSION)
def entry():
    pass


@click.command(name="mint")
@click.option('--keypair', '-k', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def mint(keypair, log_level):
    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(keypair)
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    mint_keypair = KeypairInput(keypair = Keypair())
    print("Mint_Id: ", mint_keypair.public_key)
    t_dets = await mint_nft(payer_keypair, mint_keypair, client, log_level)
    print(t_dets)
    await client.close()

@click.group(name="config")
async def config():
    pass

@click.command(name = "set")
@click.option("--program_id", "-p")
@click.option("--url", "-u")
@click.option("--keypair", "-k")
def set(program_id, url, keypair):
    assert program_id or url or keypair, "No options specified. Use --help for more information."
    if program_id:
        try:
            program_pubkey = parse_pubkey_input(program_id)
        except Exception as e:
            print("Invalid Public Key provided.")
            return
        set_program_id(program_pubkey.public_key.__str__())
        print("Program ID set to: ", program_pubkey.public_key)
    if url:
        if url.lower() == "mainnet" or url.lower() == "testnet" or url.lower() == "devnet":
            url = rpc_url.get_network_url(url)
        set_network(url)
        print("Network set to: ", url)
    if keypair:
        set_keypair_path(keypair)
        print("Keypair set to: ", keypair)
    if not program_id and not url and not keypair:
        print("No options specified. Use --help for more information.")
        return
    print("Config set successfully.")

@click.command(name = "get")
def get():
    print("\nProgram ID: ", get_program_id())
    print("Network: ", get_network())
    print("Keypair: ", get_keypair_path())
    print("\nConfig retrieved successfully.")

config.add_command(set)
config.add_command(get)

@click.command(name="init_rebalance")
@click.option('--keypair', '-k', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def initialize_rebalancing(keypair, log_level):
    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(keypair)
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    t_dets = await init_rebalance(payer_keypair, client, log_level)
    print(t_dets)
    await client.close()

@click.command(name="finalize_rebalance")
@click.option('--keypair', '-k', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def finalize_rebalancing(keypair, log_level):
    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(keypair)
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    t_dets = await finalize_rebalance(payer_keypair, client, log_level)
    print(t_dets)
    await client.close()

@click.command(name="init")
@click.option('--keypair', '-k', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def ingl(keypair, log_level):
    init_commission = click.prompt("Enter the Commission to be set for the validator: ", type=int)
    max_primary_stake = click.prompt("Enter the maximum primary stake to be set for the validator: ", type=int)
    nft_holders_share = click.prompt("Enter the NFT Holders Share to be set for the validator: ", type=int)
    initial_redemption_fee = click.prompt("Enter the Initial Redemption Fee to be set for the validator: ", type=int)
    is_validator_switchable = click.prompt("Is the validator switchable? (y/n): ", type=bool)
    unit_backing = click.prompt("Enter the Unit Backing to be set for the validator: ", type=int)
    redemption_fee = click.prompt("Enter the Redemption Fee to be set for the validator: ", type=int)
    proposal_quorum = click.prompt("Enter the Proposal Quorum to be set for governance proposals: ", type=int)
    creator_royalty = click.prompt("Enter the Creator Royalty to be set for the validator: ", type=int)
    rarities = [7000, 2900, 100]#TODO: Make this dynamic
    rarity_name = ["common", "rare", "epic"]#TODO: Make this dynamic
    twitter_handle = click.prompt("Enter the Twitter handle of the validator: ", type=str)
    discord_invite = click.prompt("Enter the Discord Invite of the validator: ", type=str)
    validator_name = click.prompt("Enter the Name of the validator: ", type=str)
    collection_uri = click.prompt("Enter the Collection URI of the validator: ", type=str)
    website = click.prompt("Enter the Website of the validator: ", type=str)


    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(keypair)
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    t_dets = await ingl_init(payer_keypair, init_commission, max_primary_stake, nft_holders_share, initial_redemption_fee, is_validator_switchable, unit_backing, redemption_fee, proposal_quorum, creator_royalty, rarities, rarity_name, twitter_handle, discord_invite, validator_name, collection_uri, website, client, log_level,)
    print(t_dets)
    await client.close()


@click.command(name="process_rewards")
@click.option('--keypair', '-k', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def process_vote_account_rewards(keypair, log_level):
    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(keypair) 
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    t_dets = await process_rewards(payer_keypair, client, log_level)
    print(t_dets)
    await client.close()

@click.command(name='create_vote_account')
@click.option('--val_keypair', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def process_create_vote_account(val_keypair, log_level):
    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(f"./{val_keypair}")
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    t_dets = await create_vote_account(payer_keypair, client, log_level)
    print(t_dets)
    await client.close()

@click.command(name='delegate_gem')
@click.argument('mint_id')
@click.argument('vote_account')
@click.option('--keypair', '-k', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def process_delegate_gem(keypair, mint_id, vote_account, log_level):
    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(keypair)
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    try:
        mint_pubkey = parse_pubkey_input(mint_id)
    except Exception as e:
        print("Invalid Public Key provided.")
        return
    try:
        vote_account_pubkey = parse_pubkey_input(vote_account)
    except Exception as e:
        print("Invalid Public Key provided.")
        return
    t_dets = await delegate_nft(payer_keypair, mint_pubkey, vote_account_pubkey, client, log_level)
    print(t_dets)
    await client.close()

@click.command(name = 'undelegate_gem')
@click.argument('mint_id')
@click.option('--keypair', '-k', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def process_undelegate_gem(keypair, mint_id, log_level):
    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(keypair)
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    try:
        mint_pubkey = parse_pubkey_input(mint_id)
    except Exception as e:
        print("Invalid Public Key provided.")
        return
    gem_account_pubkey, _gem_account_bump = PublicKey.find_program_address([bytes(ingl_constants.GEM_ACCOUNT_CONST, 'UTF-8'), bytes(mint_pubkey.public_key)], get_program_id())
    gem_account = await client.get_account_info(gem_account_pubkey)
    gem_account_data = gem_account.value.data
    funds_location_data = gem_account_data[20:52]
    funds_location_pubkey = PubkeyInput(pubkey = PublicKey(funds_location_data))
    # print(funds_location_pubkey)
    t_dets = await undelegate_nft(payer_keypair, mint_pubkey, funds_location_pubkey, client, log_level)
    print(t_dets)
    await client.close()

@click.command(name='init_governance')
@click.argument('mint_id')
@click.option('--keypair', '-k', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def process_create_upgrade_proposal(keypair, mint_id, log_level):
    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(keypair)
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    try:
        mint_pubkey = parse_pubkey_input(mint_id)
    except Exception as e:
        print("Invalid Public Key provided for mint.")
        return

    governed = ["Validator Name", "Program Upgrade"]
    for i in range(len(governed)):
        print(f"{i} : {governed[i]}")
    numeration = click.prompt("Enter the number of the governed item", type=int)
    if numeration not in range(len(governed)):
        print("Invalid Input")
        return
    if numeration == 0:
        value = click.prompt("Enter the new validator name: ", type=str)
        t_dets = await init_governance(payer_keypair, mint_pubkey, client, config_account_type = ConfigAccountType.enum.ValidatorName(value), log_level = log_level)
    elif numeration == 1:
        try:
            buffer_address = parse_pubkey_input(click.prompt("Enter the buffer address: ", type=str)).public_key
        except Exception as e:
            print("Invalid Public Key provided for buffer.")
            return
        code_link = click.prompt("Enter the code link: ", type=str)

        t_dets = await init_governance(payer_keypair, mint_pubkey, client, GovernanceType.enum.ProgramUpgrade(buffer_account = buffer_address, code_link = code_link), log_level)
    print(t_dets)
    await client.close()

@click.command(name='vote_upgrade_proposal')
@click.argument('vote')
@click.argument('proposal')
@click.argument('vote_account')
@click.option('--keypair', '-k', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def process_vote_upgrade_proposal(keypair, vote, proposal, vote_account, log_level):
    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(keypair)
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    vote = parse_vote(vote)
    upgrade_proposal_pubkey, upgrade_proposal_numeration = parse_proposal(proposal)
    vote_account_key, vote_account_numeration = parse_proposal(vote_account)
    print(f"Vote_account: {vote_account_key}, Proposal_Account {upgrade_proposal_pubkey}, Vote: {'Approve' if vote else 'Dissaprove'} ");
    t_dets = await vote_governance(payer_keypair, vote, upgrade_proposal_pubkey, upgrade_proposal_numeration, vote_account_key, vote_account_numeration, client, log_level)
    print(t_dets)
    await client.close()

@click.command(name='finalize_upgrade_proposal')
@click.argument('proposal')
@click.option('--keypair', '-k', default = get_keypair_path())
@click.option('--log_level', '-l', default = 2, type=int)
async def process_finalize_upgrade_proposal(keypair, proposal, log_level):
    client = AsyncClient(rpc_url.target_network)
    client_state = await client.is_connected()
    print("Client is connected" if client_state else "Client is Disconnected")
    try:
        payer_keypair = parse_keypair_input(keypair)
    except Exception as e:
        print("Invalid Keypair Input. ")
        return
    upgrade_proposal_pubkey, upgrade_proposal_numeration = parse_proposal(proposal)
    print(f"Proposal_Account {upgrade_proposal_pubkey}");
    t_dets = await finalize_governance(payer_keypair, upgrade_proposal_pubkey, upgrade_proposal_numeration, client, log_level)
    print(t_dets)
    await client.close()    

entry.add_command(mint)
entry.add_command(initialize_rebalancing)
entry.add_command(finalize_rebalancing)
entry.add_command(ingl)
entry.add_command(process_create_vote_account)
entry.add_command(process_delegate_gem)
entry.add_command(process_undelegate_gem)
entry.add_command(process_create_upgrade_proposal)
entry.add_command(process_vote_upgrade_proposal)
entry.add_command(process_finalize_upgrade_proposal)
entry.add_command(config)
if __name__ == '__main__':
    entry()