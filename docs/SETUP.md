# VaultRaise Developer Handoff

This document contains instructions for setting up, building, testing, and deploying the VaultRaise Solana program.

## 1. Local Environment Setup

- **Rust and Cargo**: v1.79.0 (Required for Solana SBF compatibility)
- **Solana CLI**: v2.1.0
- **Anchor CLI**: v0.31.1
- **Node.js**: v20+ (for Devnet testing script)

Verified WSL tool versions:

```text
rustc 1.97.1
cargo 1.97.1
solana-cli 2.1.0
anchor-cli 0.31.1
```

To enter the WSL Ubuntu environment from Windows PowerShell:
```powershell
wsl -d Ubuntu --cd /mnt/d/01_Projects/solana-crowdfunding-platform
```

## 2. Build Instructions

To build the program and generate the IDL, run this inside WSL:

```bash
anchor build
```
Note: Wait for the `idl build` to complete. If it gets stuck downloading crates, it's normal on the first run. The compiled binary will be placed in `target/deploy/vault_raise.so`.

## 3. Test Instructions

**Unit Tests (Rust)**
Run local environment tests using `solana-program-test`:
```bash
cargo test
```
This tests all scenarios (Creation, Contribution, Withdrawal, and Refund) in a local in-memory validator without needing a live network.

**Devnet E2E Tests (TypeScript)**
To test against a live network (Devnet), run the TypeScript script from the host (Windows):
```powershell
npm install
npx tsx scripts/devnet_test.ts
```

## 4. Devnet Deployment Instructions

1. Get some Devnet SOL: `solana airdrop 5 -u devnet`
2. Make sure `Anchor.toml` is pointed to `cluster = "devnet"`.
3. Make sure the Program ID in `Anchor.toml` matches `declare_id!()` in `lib.rs`.
4. Deploy:
```bash
anchor deploy
```
5. If the program ID changes (e.g., deleted target folder), run `solana address -k target/deploy/vault_raise-keypair.json` to get the new ID, update `lib.rs` and `Anchor.toml`, then `anchor build && anchor deploy` again.

## 5. MVP Known Limitations

- **No SPL Token Support**: The program currently only accepts native SOL, not USDC or other tokens.
- **Account Closure**: Campaign accounts are not closed after completion/refund to serve as an on-chain audit trail. This leaves some rent tied up.
- **No Platform Fees**: 100% of the funds go to the creator or back to donors.
- **Over-funding Allowed**: Donors can still contribute even if the funding goal has already been reached, as long as the deadline hasn't passed.
- **Timestamp Accuracy**: Relies on Solana's `Clock::unix_timestamp`, which can vary slightly from real-world time.
