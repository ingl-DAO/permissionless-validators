# Architecture

A native Solana program (no Anchor) that fractionalizes validator ownership. Entrypoint is
`src/lib.rs` → `src/processor.rs`, which decodes an `InstructionEnum` (`src/instruction.rs`) and
dispatches into one of five process groups under `src/processes/`.

```
src/
├── lib.rs           program entrypoint
├── instruction.rs   InstructionEnum — the 18-variant instruction surface
├── processor.rs     dispatch
├── state.rs         all account layouts (Borsh)
├── utils.rs         PDA derivation, assertions, helpers
├── error.rs         InglError
└── processes/
    ├── init_processes/        instance setup
    ├── nft_processes/         the fractional shares
    ├── validator_processes/   vote account
    ├── rewards_processes/     reward flow
    └── governance_processes/  on-chain governance
python/               Click-based `ingl` CLI (pip-installable) that drives every instruction
```

## Instruction surface

`InstructionEnum` in `src/instruction.rs`:

| Group | Instructions |
| --- | --- |
| Init | `UploadUris`, `ResetUris` |
| NFT | `MintNft`, `ImprintRarity`, `DelegateNFT`, `UnDelegateNFT`, `Redeem` |
| Validator | `CreateVoteAccount` |
| Rewards | `ProcessRewards`, `InitRebalance`, `FinalizeRebalance`, `NFTWithdraw` |
| Governance | `InitGovernance`, `VoteGovernance`, `FinalizeGovernance`, `ExecuteGovernance` |
| Testing | `InjectTestingData` |

## Lifecycle

1. **Initialize** — `init_processes/init.rs` writes a `ValidatorConfig` (the instance's parameters:
   commission, max NFTs, rarity distribution, unit backing). `fractionalize_existing.rs` covers
   the case of wrapping a validator that already exists rather than creating a fresh one.
2. **Upload URIs** — `upload_uris.rs` fills a `UrisAccount` with the NFT metadata URIs, one set per
   rarity tier. Gated by an `upload_authority`. `reset_uris.rs` exists because the upload is
   chunked across several transactions and can half-fail.
3. **Mint** — `nft_processes/mint_nft.rs` mints a share NFT (Metaplex token metadata + a program
   `NftData` account), freezing it so rarity must be imprinted before it can move.
   `imprint_rarity.rs` then assigns the rarity tier. Rarity was originally sourced from
   Switchboard **VRF** and later moved to **data feeds** — simpler, cheaper, and no callback
   account to keep alive.
4. **Delegate** — `delegate_nft.rs` moves the NFT's backing SOL into the validator's stake
   (`FundsLocation` in `state.rs` tracks whether a share sits in the pool or is delegated).
   `undelegate_nft.rs` reverses it, subject to cooldown.
5. **Vote account** — `validator_processes/create_vote_account.rs` creates the vote account owned
   by a program PDA, so no single human holds the validator's authority.
6. **Rewards** — `process_rewards.rs` collects vote rewards into a `VoteReward` history.
   Distribution runs as a two-phase rebalance (`init_rebalance.rs` → `finalize_rebalance.rs`)
   because a stake account cannot be split and merged within a single epoch boundary; the split
   state lives in `RebalancingData`. `nft_withdraw.rs` pays an individual holder out.
7. **Governance** — `init_governance.rs` opens a proposal (`GovernanceType` covers config changes,
   program upgrades, and vote-account changes via `VoteAccountGovernance`), `vote_governance.rs`
   records NFT-weighted votes into `GovernanceData`, `finalize_governance.rs` closes it once the
   vote threshold and time window are met, `execute_governance.rs` applies the result.

## Notable state

All layouts are Borsh-serialized in `src/state.rs`. The ones worth knowing:

- **`ValidatorConfig`** — the instance's immutable-ish parameters.
- **`GeneralData`** — mutable counters: NFTs minted, delegated, pending rewards, epoch bookkeeping.
- **`NftData`** — per-share: rarity, `FundsLocation`, redemption state.
- **`RebalancingData`** — the in-flight state of a two-phase reward rebalance.
- **`GovernanceData`** — votes, thresholds, `date_finalized`.
- **`VoteState` / `VoteStateVersions` / `VoteState0_23_5` / `AuthorizedVoters` / `Lockout` /
  `CircBuf`** — a local mirror of Solana's own vote account layout, needed to read vote credits
  directly out of the vote account. Legacy version handling was later dropped in favour of the
  current layout only.

## Why account sizing shows up everywhere

Solana accounts are fixed-size and rent-exempt, so several instructions exist purely to compute or
grow space (`get_space` implementations across `state.rs`, the chunked URI upload). Most of the
subtle bugs in this program's history were sizing or offset bugs, not logic bugs.

## CLI

`python/src/ingl_cli.py` exposes the whole surface as Click commands — `ingl init`,
`ingl mint`, `ingl delegate`, `ingl create_vote_account`, `ingl upload_uris`,
`ingl process_rewards`, `ingl init_rebalance`, `ingl finalize_rebalance`, `ingl init_governance`,
plus `ingl config set/get`. Instruction encoding mirrors `src/instruction.rs` in
`python/src/instruction.py`.
