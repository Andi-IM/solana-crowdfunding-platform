# VaultRaise

VaultRaise is a Solana-based crowdfunding platform built with Anchor. 

It solves the common trust issues in traditional crowdfunding by utilizing a **Program Derived Address (PDA) Vault** to escrow funds. Contributions are locked on-chain and can only be withdrawn by the creator if the campaign successfully reaches its goal before the deadline. If the campaign fails, donors can securely claim their refunds.

## Core Features (MVP)

1. **Create Campaign**: Creators can launch a crowdfunding campaign with a specific funding goal and a deadline.
2. **Contribute**: Donors can send SOL to the campaign's vault. Funds are securely locked in a PDA, meaning they are not directly sent to the creator's wallet.
3. **Withdraw**: If the campaign reaches its goal and the deadline has passed, the creator can withdraw the accumulated funds.
4. **Refund**: If the campaign fails to reach its goal by the deadline, donors can claim a full refund of their contributions.

## Technical Stack

- **Blockchain**: Solana
- **Smart Contract Framework**: Anchor / `anchor-lang` (v0.32.1)
- **Language**: Rust 2021 edition

## Prerequisites

To build and test this project locally, ensure you have the following installed:

- [Rust and Cargo](https://rustup.rs/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v2.1.0)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) compatible with `anchor-lang` v0.32.1

If you are on Windows, it is highly recommended to use **WSL (Ubuntu)** for development.

## Getting Started

### 1. Clone the Repository

```bash
git clone https://github.com/Andi-IM/solana-crowdfunding-platform.git
cd solana-crowdfunding-platform
```

### 2. Build the Program

To compile the smart contract, run:

```bash
cargo build-sbf --manifest-path programs/vault_raise/Cargo.toml
```

This generates the compiled SBF program at `target/deploy/vault_raise.so`, which is also used by the Rust integration tests.

### 3. Run Tests

The Rust integration tests run the compiled SBF program through `solana-program-test`. Build the SBF artifact first, then run the tests:

```bash
cargo check
cargo build-sbf --manifest-path programs/vault_raise/Cargo.toml
cargo test
```

## Usage Flow

Typical client usage:

1. Derive the campaign PDA with `["campaign", creator, campaign_id]`.
2. Derive the vault PDA with `["vault", campaign]`.
3. Call `create_campaign(campaign_id, goal, deadline)` with a positive lamport goal and a future Unix timestamp.
4. Derive each donor contribution PDA with `["contribution", campaign, donor]`.
5. Call `contribute(amount)` before the deadline to transfer SOL into the vault PDA.
6. After the deadline, call `withdraw()` if `raised >= goal`, or `refund()` if `raised < goal`.
7. Optionally close settled accounts with `close_campaign()` or `close_refunded_contribution()` if rent recovery is preferred over retaining account data. The core MVP lifecycle does not require account closure.

## Architecture Notes

The Anchor program is split by responsibility:

- `instructions/`: account validation structs and instruction handlers.
- `state.rs`: campaign, contribution, governance, PDA seeds, and asset model.
- `events.rs`: structured Anchor events for indexers and clients.
- `errors.rs`: custom program errors.

The current funding implementation handles native SOL only. Campaign state also stores a `FundingAsset` enum with an SPL Token variant, but SPL token transfers and token-account validation are not implemented yet.

Campaign metadata uses an explicit `update_campaign_metadata()` realloc path with a bounded URI length. This keeps the base campaign state small while making future metadata updates intentional and rent-funded by the creator.

Governance is represented by a singleton PDA derived from `["governance"]`. It currently supports authority initialization and transfer as a narrow foundation for future administrative controls; no privileged campaign controls are wired to governance yet.

Common program errors:

| Error | Meaning |
| --- | --- |
| `InvalidGoal` | Campaign goal must be greater than zero. |
| `InvalidDeadline` | Campaign deadline must be in the future. |
| `CampaignEnded` | Contribution was attempted after the campaign deadline. |
| `CampaignNotEnded` | Withdraw or refund was attempted before the deadline. |
| `CampaignNotSuccessful` | Withdraw was attempted before the goal was reached. |
| `CampaignNotFailed` | Refund was attempted for a successful campaign. |
| `UnauthorizedCreator` | A non-creator tried to withdraw. |
| `AlreadyClaimed` | Campaign funds have already been withdrawn. |
| `AlreadyRefunded` | The donor contribution has already been refunded. |
| `InvalidContributionAmount` | Contribution or refund amount is zero or invalid. |
| `ArithmeticOverflow` | Lamport accounting overflowed. |

## Deployment

The program has been deployed to the Solana Devnet.

- **Devnet Program ID**: `GeYMy79EJmUs8japokaVcadb2RRs6vv7c4xYE2fbjkQW`
- **Explorer Link**: [View on Solana Explorer](https://explorer.solana.com/address/GeYMy79EJmUs8japokaVcadb2RRs6vv7c4xYE2fbjkQW?cluster=devnet)

## Documentation

For more in-depth information about the project's architecture, design decisions, and tasks, please refer to the `docs/` directory:

- [Project Context & Architecture](docs/PROJECT_CONTEXT.md)
- [Jira Tasks Breakdown](docs/JIRA_TASKS.md)
- [Setup & Environment Notes](docs/SETUP.md)
- [Task Tracking](docs/TASK_TRACKING.md)

## License

This project is open-source and available under the MIT License.
