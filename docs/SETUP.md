# VaultRaise Developer Handoff

This document contains instructions for setting up, building, testing, and deploying the VaultRaise Solana program.

## 1. Local Environment Setup

- **Rust and Cargo**: Rust 2021 edition compatible toolchain
- **Solana CLI**: v2.1.0
- **Anchor CLI / anchor-lang**: v0.32.1-compatible
- **Node.js**: v20+ (for Devnet testing script)

Verified WSL tool versions:

```text
solana-cli 2.1.0
anchor-lang 0.32.1
```

To enter the WSL Ubuntu environment from Windows PowerShell:
```powershell
wsl -d Ubuntu --cd /mnt/d/01_Projects/solana-crowdfunding-platform
```

## 2. Build Instructions

To build the SBF program used by local integration tests, run this inside WSL:

```bash
cargo build-sbf --manifest-path programs/vault_raise/Cargo.toml
```

The compiled binary will be placed in `target/deploy/vault_raise.so`.

If you also need to regenerate Anchor IDL artifacts, use an Anchor CLI version compatible with `anchor-lang 0.32.1`, then run:

```bash
anchor build
```

## 3. Test Instructions

**Unit Tests (Rust)**
Run local environment tests using `solana-program-test`. These tests load `target/deploy/vault_raise.so`, so build the SBF artifact first:
```bash
cargo build-sbf --manifest-path programs/vault_raise/Cargo.toml
cargo test
```
This tests all scenarios (Creation, Contribution, Withdrawal, and Refund) in a local in-memory validator without needing a live network.

The tests verify both transaction outcomes and on-chain account state for `Campaign` and `Contribution` accounts.

**Devnet E2E Tests (TypeScript)**
To test against a live network (Devnet), run the TypeScript script from the host (Windows):
```powershell
npm install
npx tsx scripts/devnet_test.ts
```

## 4. Usage And Integration Notes

PDA derivation:

```text
campaign = ["campaign", creator.key(), campaign_id]
vault = ["vault", campaign.key()]
contribution = ["contribution", campaign.key(), donor.key()]
```

Instruction sequence:

1. `create_campaign(campaign_id, goal, deadline)` initializes the campaign state and vault PDA.
2. `update_campaign_metadata(metadata_uri)` optionally reallocates campaign metadata within the configured URI limit.
3. `contribute(amount)` transfers SOL from donor to vault and records the donor contribution.
4. `withdraw()` transfers vault SOL to the creator after the deadline if `raised >= goal`.
5. `refund()` transfers a donor's contribution back after the deadline if `raised < goal`.
6. `close_campaign()` and `close_refunded_contribution()` close settled accounts and return rent.

Architecture layout:

```text
programs/vault_raise/src/
  lib.rs                    Anchor entrypoint
  errors.rs                 custom errors
  events.rs                 structured events
  state.rs                  accounts, PDA seeds, asset abstraction
  instructions/
    campaign.rs             campaign create/update/withdraw/close
    contribution.rs         contribute/refund/close contribution
    governance.rs           governance initialization and authority transfer
```

Design extension points:

- `FundingAsset` stores `NativeSol` now and reserves a `SplToken { mint }` variant for SPL token vault instructions.
- `Campaign::realloc_space()` and `update_campaign_metadata()` define the bounded account reallocation path.
- `Governance` is a singleton PDA for future upgrade/admin controls.

Common errors:

| Error | Typical cause |
| --- | --- |
| `InvalidGoal` | `goal == 0`. |
| `InvalidDeadline` | `deadline <= Clock::get()?.unix_timestamp`. |
| `CampaignEnded` | Contribution after deadline. |
| `CampaignNotEnded` | Withdraw or refund before deadline. |
| `CampaignNotSuccessful` | Withdraw before funding goal is met. |
| `CampaignNotFailed` | Refund attempted on a successful campaign. |
| `UnauthorizedCreator` | Withdraw signer does not match `campaign.creator`. |
| `AlreadyClaimed` | Withdraw attempted more than once. |
| `AlreadyRefunded` | Refund attempted more than once for the same contribution. |
| `InvalidContributionAmount` | Contribution amount is zero or refund contribution amount is zero. |
| `ArithmeticOverflow` | Lamport addition overflowed. |

## 5. Devnet Deployment Instructions

1. Get some Devnet SOL: `solana airdrop 5 -u devnet`
2. Make sure `Anchor.toml` is pointed to `cluster = "devnet"`.
3. Make sure the Program ID in `Anchor.toml` matches `declare_id!()` in `lib.rs`.
4. Build and deploy:
```bash
cargo build-sbf --manifest-path programs/vault_raise/Cargo.toml
anchor deploy
```
5. If the program ID changes (e.g., deleted target folder), run `solana address -k target/deploy/vault_raise-keypair.json` to get the new ID, update `lib.rs` and `Anchor.toml`, then `anchor build && anchor deploy` again.

## 6. MVP Known Limitations

- **No SPL Token Support**: The program currently only accepts native SOL, not USDC or other tokens.
- **Account Closure**: Campaign accounts are not closed after completion/refund to serve as an on-chain audit trail. This leaves some rent tied up.
- **No Platform Fees**: 100% of the funds go to the creator or back to donors.
- **Over-funding Allowed**: Donors can still contribute even if the funding goal has already been reached, as long as the deadline hasn't passed.
- **Timestamp Accuracy**: Relies on Solana's `Clock::unix_timestamp`, which can vary slightly from real-world time.
