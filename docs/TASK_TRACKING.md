# Task Tracking

## VR-001 - Initialize Solana/Anchor Project Structure

Status: Done with environment note

Started: 2026-07-17
Completed: 2026-07-17

Scope:

- Scaffold Anchor project structure.
- Add Rust workspace and program crate.
- Add Anchor configuration.
- Add placeholder test harness.
- Keep AI-related local files ignored.

Evidence:

- `.agents/` and `.codex/` are ignored through `.gitignore`.
- `Anchor.toml` exists.
- Root `Cargo.toml` workspace exists.
- Program crate exists at `programs/vault_raise`.
- `cargo check` completed successfully.

Environment Note:

- `anchor` CLI is available in WSL through AVM.
- `solana` CLI is available in WSL.
- Rust and Cargo are available in WSL through rustup.

Follow-up Environment Work:

- WSL Ubuntu is available as the default WSL 2 distribution.
- Project access from WSL verified at `/mnt/d/01_Projects/solana-crowdfunding-platform`.
- Added `scripts/enter-wsl.ps1` to open Ubuntu WSL directly in the project folder.
- Installed Rust/Cargo in WSL user space.
- Installed Solana CLI in WSL user space.
- Installed AVM and activated Anchor CLI `0.31.1` in WSL.
- Added release overflow checks to satisfy Anchor build requirements.
- Downgraded locked `blake3` dependency to avoid `cpufeatures 0.3.0`, which requires edition 2024 and is incompatible with Solana SBF Cargo 1.79.
- Pinned locked `solana-program` dependency family to `2.1.0` to align with Anchor CLI `0.31.1` and the Solana SBF build toolchain.
- Downgraded locked `borsh` dependency to `1.5.5` to avoid edition-2024 transitive dependencies during SBF build.
- Downgraded locked `proc-macro-crate` to `3.2.0` and `indexmap` to `2.7.1` for compatibility with Solana SBF Cargo 1.79.
- Downgraded locked `zeroize` to `1.8.2` for compatibility with Solana SBF Cargo 1.79.
- Downgraded locked `unicode-segmentation` to `1.12.0` for compatibility with Solana SBF Rust 1.79.
- Added program `idl-build` feature required by Anchor CLI `0.31.1`.
- `anchor build` completed successfully from WSL.

Follow-up Cleanup:

- Removed Node.js/TypeScript scaffold files from the project.
- Removed Yarn/ts-mocha test script from `Anchor.toml`.

## VR-002 - Define Program Accounts And Error Types

Status: Done

Started: 2026-07-17
Completed: 2026-07-17

Scope:

- Define `Campaign` account.
- Define `Contribution` account.
- Define custom program errors.
- Keep task limited to account and error definitions only.

Evidence:

- Added `Campaign` account with creator, goal, raised, deadline, claimed, bump, and vault bump fields.
- Added `Contribution` account with campaign, donor, amount, refunded, and bump fields.
- Added `VaultRaiseError` enum for expected validation and safety failures.
- `cargo check` completed successfully from WSL.
- `anchor build` completed successfully from WSL.

## VR-003 - Implement Create Campaign Instruction

Status: Done

Started: 2026-07-17
Completed: 2026-07-17

Scope:

- Implement `create_campaign` instruction.
- Accept `_campaign_id`, `goal`, and `deadline`.
- Validate `goal > 0` and future `deadline`.
- Initialize `Campaign` account state.
- Write log event for campaign creation.

Evidence:

- `lib.rs` contains `create_campaign` function and `CreateCampaign` accounts struct.
- Validations use `require!` macro with `VaultRaiseError`.
- Campaign account initialized with correct values.
- `cargo check` completed successfully in WSL without unused variable warnings for `_campaign_id`.
