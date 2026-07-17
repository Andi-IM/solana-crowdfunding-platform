# VaultRaise Setup

This project is scaffolded as an Anchor-based Solana program.

## Required Tools

- Rust and Cargo
- Solana CLI
- Anchor CLI
- Node.js package manager compatible with Anchor tests

Verified WSL tool versions:

```text
rustc 1.97.1
cargo 1.97.1
solana-cli 2.1.0
anchor-cli 0.31.1
avm 1.1.2
```

## Current Scaffold

- `Anchor.toml`: Anchor workspace configuration.
- `Cargo.toml`: Rust workspace configuration.
- `programs/vault_raise`: Solana program crate.
- `tests/vault_raise.ts`: placeholder TypeScript test harness.

## Basic Commands

```powershell
cargo check
anchor build
anchor test
```

## Accessing The Project From WSL

WSL is available on this machine with Ubuntu as the default distribution.

From PowerShell, open the project in WSL with:

```powershell
.\scripts\enter-wsl.ps1
```

Manual equivalent:

```powershell
wsl -d Ubuntu --cd /mnt/d/01_Projects/solana-crowdfunding-platform
```

Inside WSL, the project path is:

```bash
/mnt/d/01_Projects/solana-crowdfunding-platform
```

## Notes

- `anchor` and `solana` are available in WSL Ubuntu after sourcing the user profile.
- The current `declare_id!()` and `Anchor.toml` program ID use the placeholder system program address. Replace them with the real program ID before deployment.
- Some lockfile dependencies are intentionally pinned to versions compatible with Solana SBF Rust/Cargo 1.79 used by Anchor CLI `0.31.1`.
