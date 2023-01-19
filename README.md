<p align="center">
  <a href="" rel="noopener">
 <img src="images/logo.png" alt="Project logo"></a>
</p>
<h3 align="center">
Fractionalizing Validator Creation and Ownership</h3>
<br />

## 
## Creating your Fractionalized Validator Instance.

## ⛓️ Prerequisites.
We currently only recommend using linux.

#### Install Solana CLI:
```
sh -c "$(curl -sSfL https://release.solana.com/v1.14.12/install)"
```
#### Install Ingl CLI:
```
pip install ingl
```
#### Install Cargo-x:
```
cargo install cargo-x
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
solana-keygen new -o ~\.config\solana\id.json
solana config set --keypair ~\.config\solana\id.json
```
#### Set the validator id
```
ingl config set -k ~\.config\solana\id.json
```
#### On DEVNET airdrop some deployment sol, ideally >10. 
On mainnet, fund the keypair
```
solana airdrop 2
```
#### Compile and deploy the program (on devnet), then set the program upgrade authority to the governance pda
```
cargo-x bdau
```
#### Initialize the program instance
``` 
Ingl Init
```
Fill in all the prompted fields.
#### Initialize the validator
```
ingl create_vote_account
```
#### Upload the image URIS
``` 
ingl upload_uris  uris_path.json
```
#### Should in case any thing fails, then reset the URIS and Reupload
``` 
ingl reset_uris
ingl upload_uris  uris_path.json
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
    --identity ~\.config\solana\id.json \
    --vote-account 'vote_account' \
    --known-validator dv1ZAGvdsz5hHLwWXsVnM94hWf1pjbKVau1QVkaMJ92 \
    --known-validator dv2eQHeP4RFrJZ6UeiZWoc3XTtmtZCUKxxCApCDcRNV \
    --known-validator dv4ACNkpYPcE3aKmYDqZm9G5EB3J4MRoeE7WNDRBVJB \
    --known-validator dv3qDFk1DTF36Z62bNvrCXe9sKATA6xvVy6A798xxAS \
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
    --limit-ledger-size
```