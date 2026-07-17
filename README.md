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
- **Smart Contract Framework**: Anchor (v0.31.1)
- **Language**: Rust (v1.79.0)

## Prerequisites

To build and test this project locally, ensure you have the following installed:

- [Rust and Cargo](https://rustup.rs/) (v1.79.0 recommended)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v2.1.0)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) (v0.31.1)

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
anchor build
```

This will generate the IDL and the compiled program in the `target/` directory.

### 3. Run Tests

To run the unit tests and ensure everything is working correctly, you can use:

```bash
cargo check
cargo test
```

## Documentation

For more in-depth information about the project's architecture, design decisions, and tasks, please refer to the `docs/` directory:

- [Project Context & Architecture](docs/PROJECT_CONTEXT.md)
- [Jira Tasks Breakdown](docs/JIRA_TASKS.md)
- [Setup & Environment Notes](docs/SETUP.md)
- [Task Tracking](docs/TASK_TRACKING.md)

## License

This project is open-source and available under the MIT License.
