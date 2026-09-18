<p align="center">
  <img src="images/logo.png" alt="Project logo" width="50%">
</p>
<h3 align="center">
Fractionalizing Validator Creation and Ownership</h3>
<br />

## Project status

> **Archived / devnet-only.** This program was built and tested on **Solana devnet**. It was never
> deployed to mainnet-beta, and it is not maintained. The instructions below are pinned to
> **Solana CLI v1.14.12** (March 2023) and will not work unmodified against current Solana releases.

**What it does.** Fractionalizes the creation and ownership of a Solana validator. Instead of
requiring one operator to supply the full stake and hardware, the program lets many participants
buy in — from roughly 1 SOL — via NFTs that represent a share of the validator, delegate that
stake to a program-owned vote account, and share in the validator's rewards. Governance over the
validator's configuration is itself on-chain and NFT-weighted.

**Scope of what was proven.** The full lifecycle runs on devnet: deploy the program → initialize
the instance → mint and delegate NFTs → create the vote account → run the validator → process and
rebalance rewards → govern changes. Mainnet was out of scope: it requires real capital at stake,
production validator hardware, and an audit — none of which applied to this project.

**What would differ on mainnet.** Real economics (stake-weighted rewards, delinquency and
commission risk), a security audit of the rebalancing and governance paths, hardened key
custody for the upgrade and withdraw authorities, and validator hardware meeting mainnet's
substantially higher CPU/RAM/NVMe bar.

### Contributors

| Area | Contributors |
| --- | --- |
| On-chain program (Rust) | [@kamda-cyrial](https://github.com/kamda-cyrial) (98 commits), [@marcjazz](https://github.com/marcjazz) (37), [@artemesian](https://github.com/artemesian) (3) |
| Client ([permissionless-validators-client](https://github.com/ingl-DAO/permissionless-validators-client)) | [@marcjazz](https://github.com/marcjazz) (213 commits), [@Tchakoumi](https://github.com/Tchakoumi) (167), [@artemesian](https://github.com/artemesian) (54) |

See [ARCHITECTURE.md](ARCHITECTURE.md) for a map of the program.

### Related

[**solana-validator-k8s**](https://github.com/ingl-DAO/solana-validator-k8s) — the operational
counterpart. This repository is the on-chain program that *creates* a validator; that one runs a
Solana node on Kubernetes, with a portable Helm chart, a disposable Terraform layer, and a
documented contract between them. Together they cover both halves: the protocol-level work of
managing vote accounts and stake, and the infrastructure work of actually operating the node.

---


## Creating your Fractionalized Validator Instance.

## ⛓️ Prerequisites.

We currently only recommend using ubuntu 20.04.

#### Install Solana CLI:

```
sh -c "$(curl -sSfL https://release.solana.com/v1.14.12/install)"
```

#### Installing Pip

```
sudo apt update
```

```
sudo apt upgrade
```

```
sudo apt install python3-pip
```

#### Installing Venv

```
sudo apt install python3-venv
```

#### Creating a virtual environment called isol

```
python3 -m venv isol
```

#### activate the virtual environment

```
source isol/bin/activate
```

#### Install Ingl CLI:

```
sudo pip install ingl
```

#### Install rust:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

```
sudo apt install build-essential
```

#### Install Cargo-x:

```
cargo install cargo-x
```
#### Installing Git:
```
sudo apt install git
```

#### Clone this repo:
```

git clone https://github.com/ingl-DAO/permissionless-validators

```

## 🎈Deploying your validator instance's program.

#### Set your terminals current working directory to the cloned repo's eldest folder:

```

cd permissionless-validators

```

#### Generate the validator instance's program keypair.

```

cargo-x nda
ingl config set -p deploy/keypair.json

```

#### Generate and set the deploying authority key

```

solana-keygen new -o ~/.config/solana/id.json

```

```

solana config set --keypair ~/.config/solana/id.json

```

#### Set the validator id

```

ingl config set -k ~/.config/solana/id.json

```
#### Switch to devnet
```

solana config set --url devnet

```
#### On DEVNET airdrop some deployment sol, ideally >12.
On mainnet, fund the keypair

```

solana airdrop 2

```
#### Repeat the command above until a balance of 12 sol or more

<!-- #### Compile and deploy the program (on devnet), then set the program upgrade authority to the governance pda
cargo-x bdau -->

#### Compile and deploy the program (on devnet),
```

cargo-x bda

```

#### Initialize the program instance (ensure the signer is the upgrade authority of the program)
```

ingl init --keypair <path to upgrade_authority>

```

Fill in all the prompted fields.

#### Initialize the validator

```

ingl create_vote_account

```

#### Upload the image URIS

```

ingl upload_uris uris_path.json

```

#### Should in case any thing fails, then reset the URIS and Reupload

```

ingl reset_uris
ingl upload_uris uris_path.json

```

<br />
<h3 align="center">
🥳🎉👏🥳🎉👏🥳🎉👏🥳🎉👏🥳🎉👏🥳🎉👏🥳🎉👏🥳🎉👏🥳🎉👏🥳🎉👏</h3>

## You Have Succesfully Created a Validator. Now let's run it.

### Let's do some system tuning:

```

sudo $(command -v solana-sys-tuner) --user $(whoami) > sys-tuner.log 2>&1 &

```

### Now lets find out what the created vote_account is:

```

ingl get_vote_pubkey

```

### Now lets use the found vote_account and our validator_id(~\.config\solana\id.json) to validate transactions

replace the 'vote_account' by the key gotten from the instruction above

```

solana-validator \
 --identity ~/.config/solana/id.json \
 --vote-account 'vote_account' \
 --known-validator dv4ACNkpYPcE3aKmYDqZm9G5EB3J4MRoeE7WNDRBVJB \
 --known-validator dv2eQHeP4RFrJZ6UeiZWoc3XTtmtZCUKxxCApCDcRNV \
 --only-known-rpc \
 --ledger ledger \
 --rpc-port 8899 \
 --dynamic-port-range 8000-8020 \
 --entrypoint entrypoint.devnet.solana.com:8001 \
 --entrypoint entrypoint2.devnet.solana.com:8001 \
 --entrypoint entrypoint3.devnet.solana.com:8001 \
 --entrypoint entrypoint4.devnet.solana.com:8001 \
 --entrypoint entrypoint5.devnet.solana.com:8001 \
 --expected-genesis-hash EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG \
 --wal-recovery-mode skip_any_corrupted_record \
 --limit-ledger-size\
 --log -

```