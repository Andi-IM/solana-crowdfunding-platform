# Solana Crowdfunding Platform - Project Context

## Document Status

This document is the initial source of context before program implementation begins. Its goal is to keep the application's direction focused on escrow-based crowdfunding on Solana: donor funds are locked until the campaign meets its success or failure conditions.

No final decision has been made for the project name. The temporary working name is:

**VaultRaise**

Reasoning: this name emphasizes the two main values of the product, which are that funds enter a locked vault and can only be withdrawn when the campaign rules are met.

Alternative branding for consideration:

- **GoalVault**: clear and directly explains that funds are locked until the goal is met.
- **PledgeLock**: emphasizes that pledges/donations have not yet been directly received by the creator.
- **CrowdVault**: simple, suitable for a general crowdfunding platform.
- **MilestoneVault**: suitable if it later expands to milestone-based campaigns.
- **TrustRaise**: highlights trust and transparency for donors.

For the MVP phase, use the working name **VaultRaise** in documentation and internal comments until a final name is chosen.

## Problem Statement

Traditional crowdfunding often suffers from trust issues:

- Donors want to donate without funds being directly received by the creator before the campaign conditions are met.
- Creators need a clear claim mechanism if the funding target is reached.
- Donors need an automatic or verifiable refund if the target is not reached.
- All parties need on-chain proof that funds are locked until the campaign conditions are fulfilled.

This platform solves these problems using a Solana Program that stores contribution funds in a vault PDA, instead of directly in the creator's wallet.

## MVP Goals

The MVP only covers four main actions:

1. Creating a campaign.
2. Making a SOL contribution to a campaign.
3. Creator withdrawing funds if the campaign is successful.
4. Donor claiming a refund if the campaign fails.

Not included in the MVP:

- SPL Tokens.
- Phased or milestone campaigns.
- Donor voting.
- Platform fees.
- A complete frontend.
- KYC or legal identity of the creator.
- Campaign moderation.

## Terminology

- **Campaign**: The crowdfunding data created by the creator.
- **Creator**: The wallet that creates the campaign and has the right to withdraw if the campaign is successful.
- **Donor**: A wallet that makes a contribution to the campaign.
- **Vault**: The program's PDA that holds the SOL contributions.
- **Contribution**: The contribution data per donor per campaign.
- **Goal**: The campaign funding target in lamports.
- **Deadline**: The Unix timestamp when the campaign ends.
- **Raised**: The total lamports that have been contributed.
- **Claimed**: A flag indicating that successful campaign funds have been withdrawn by the creator.

## Account Model

### Campaign Account

Stores the main state of the campaign.

Initial fields:

```text
creator: Pubkey
goal: u64
raised: u64
deadline: i64
claimed: bool
bump: u8
vault_bump: u8
```

Notes:

- `goal` is stored in lamports.
- `deadline` uses a unix timestamp with the `i64` type, following Solana's `Clock::unix_timestamp`.
- `raised` starts at `0`.
- `claimed` starts at `false`.
- `bump` and `vault_bump` are stored so PDA signing is more explicit.

### Vault PDA

The Vault is a PDA that stores the campaign's SOL.

The Vault does not need to store complex data. Implementation options:

- A PDA system account with just lamports.
- A PDA derived from the campaign.

Suggested seed:

```text
vault = ["vault", campaign.key()]
```

### Contribution Account

The Contribution is needed so that refunds can be processed correctly per donor.

Initial fields:

```text
campaign: Pubkey
donor: Pubkey
amount: u64
refunded: bool
bump: u8
```

Suggested seed:

```text
contribution = ["contribution", campaign.key(), donor.key()]
```

Reasons the contribution account is required:

- The program must know the refund amount for the donor.
- The program must prevent double refunds.
- The program must still be able to accept multiple contributions from the same donor by adding to `amount`.

## Rust/Solana Technical Specifications

This section serves as the minimum technical reference for implementing the Solana program.

### Campaign Data Structure

The main campaign state is stored in the `Campaign` account.

```rust
pub struct Campaign {
    pub creator: Pubkey,    // Who created this
    pub goal: u64,          // Target amount
    pub raised: u64,        // Current amount
    pub deadline: i64,      // When it ends
    pub claimed: bool,      // Already withdrawn?
}
```

Implementation notes:

- `creator` is the wallet of the campaign creator and the only signer allowed to withdraw.
- `goal` and `raised` use lamports.
- `deadline` uses `i64` because `Clock::unix_timestamp` in Solana is also of type `i64`.
- `claimed` prevents double withdrawals.
- The final implementation will likely need to add `bump` or other metadata if using Anchor PDA accounts.

### The Vault

Donations must not be sent directly to the creator. All contribution funds must go to a Program Derived Address (PDA) acting as a vault controlled by the program.

```rust
// Derive the vault address
let (vault_pda, bump) = Pubkey::find_program_address(
    &[b"vault", campaign_account.key.as_ref()],
    program_id
);

// Later, when transferring FROM the vault, use invoke_signed:
invoke_signed(
    &system_instruction::transfer(vault_pda, recipient, amount),
    &[vault_account, recipient_account, system_program],
    &[&[b"vault", campaign_account.key.as_ref(), &[bump]]]
)?;
```

Explanation:

- A PDA is an account whose address is derived deterministically from a seed and the `program_id`.
- PDAs do not have private keys.
- The program can "sign" for a PDA using the same seed via `invoke_signed`.
- With a PDA vault, the creator cannot take the funds before the withdraw conditions are valid.
- The vault seed used:

```text
["vault", campaign_account.key]
```

Rules for using the vault:

- During `contribute`, the transfer is made from the donor to the vault PDA.
- During `withdraw`, the transfer is made from the vault PDA to the creator using `invoke_signed`.
- During `refund`, the transfer is made from the vault PDA to the donor using `invoke_signed`.

### Getting Current Time

The program must use the on-chain time from the Solana `Clock`, not the timestamp from the client.

```rust
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

let clock = Clock::get()?;
let current_time = clock.unix_timestamp;
```

Usage:

- `create_campaign`: valid if `deadline > current_time`.
- `contribute`: valid if `current_time < campaign.deadline`.
- `withdraw`: valid if `current_time >= campaign.deadline`.
- `refund`: valid if `current_time >= campaign.deadline`.

## Program Instructions

### 1. Create Campaign

The creator creates a new campaign.

Input:

```text
goal: u64
deadline: i64
```

Validation:

- `deadline` must be greater than the current unix timestamp.
- `goal` should be greater than `0`.

State stored:

```text
creator = creator.key()
goal = goal
deadline = deadline
raised = 0
claimed = false
```

Log:

```text
Campaign created: goal={goal}, deadline={deadline}
```

Design notes:

- The Campaign PDA needs a stable seed. If one creator can make multiple campaigns, use a campaign id or counter.
- For the simplest MVP, a campaign can be created with the seed:

```text
campaign = ["campaign", creator.key(), campaign_id]
```

`campaign_id` can be a `u64` from the input or a timestamp provided by the client, but it's safer if the final design chooses one explicit approach before coding.

### 2. Contribute

The donor sends SOL to the campaign vault.

Input:

```text
amount: u64
```

Validation:

- `amount` must be greater than `0`.
- The current time should be less than the `deadline` so campaigns that have ended don't receive new contributions.
- The campaign has not been `claimed`.

Logic:

- Transfer SOL from the donor to the campaign vault PDA.
- Update `campaign.raised += amount`.
- Create or update the `Contribution Account`.
- If the donor has contributed before, add `amount` to the previous contribution.

Log:

```text
Contributed: {amount} lamports, total={raised}
```

Important notes:

- Use checked arithmetic to avoid overflows on `raised += amount`.
- Do not transfer directly to the creator.

### 3. Withdraw

The creator claims the funds if the campaign is successful.

Conditions:

- `campaign.raised >= campaign.goal`
- Current time `>= campaign.deadline`
- Caller is `campaign.creator`
- Campaign has never been claimed

Logic:

- Transfer all available SOL in the vault to the creator.
- Mark `campaign.claimed = true`.

Log:

```text
Withdrawn: {amount} lamports
```

Important notes:

- The amount withdrawn should be calculated from the available vault lamports, while maintaining rent exemption if the vault is an account that must stay alive.
- If the vault is a system account PDA without data, the close/transfer design needs to ensure it follows the Anchor pattern being used.
- After `claimed = true`, refunds cannot be made.

### 4. Refund

The donor takes back their contribution if the campaign fails.

Conditions:

- `campaign.raised < campaign.goal`
- Current time `>= campaign.deadline`
- The contribution belonging to the donor exists.
- The contribution has not been refunded.
- The campaign has not been claimed.

Logic:

- Transfer the donor's contribution amount from the vault back to the donor.
- Mark `contribution.refunded = true`.
- Set `contribution.amount = 0` after the transfer so the state is consistent.

Log:

```text
Refunded: {amount} lamports
```

Important correction note:

- A refund should transfer funds **from the vault to the donor**, not from the donor to the vault.
- If all donors have refunded, the campaign account can be left as is for an audit trail or closed in advanced features.

## State Machine

```text
Draft/Created
  -> Active until deadline
  -> Ended Successful if raised >= goal
  -> Ended Failed if raised < goal

Ended Successful
  -> Withdrawn by creator

Ended Failed
  -> Refunded per donor
```

Rules:

- Contribute is only allowed before the deadline.
- Withdraw is only allowed after the deadline and if the goal is met.
- Refund is only allowed after the deadline and if the goal is not met.
- A successful campaign cannot refund.
- A failed campaign cannot withdraw.
- A claimed campaign cannot receive contributions or refunds.

## Security And Correctness Notes

- Use a PDA for the vault so the creator cannot take funds before the conditions are met.
- Use the `Clock` sysvar to read the current unix timestamp.
- Use checked arithmetic for all lamport additions.
- Validate all signers and account ownerships.
- Ensure the contribution account always matches the campaign and donor.
- Prevent double refunds using `refunded`.
- Prevent double withdrawals using `claimed`.
- Do not trust the timestamp from the client for time validation.
- Do not allow a campaign to receive contributions after the deadline.
- Consider lamport overflow and underflow.

## Initial Error Cases

Suggested error names:

```text
InvalidGoal
InvalidDeadline
CampaignEnded
CampaignNotEnded
CampaignNotSuccessful
CampaignNotFailed
UnauthorizedCreator
AlreadyClaimed
AlreadyRefunded
InvalidContributionAmount
ArithmeticOverflow
InsufficientVaultBalance
```

## Event / Log Strategy

Minimum according to requirements:

```text
Campaign created: goal={goal}, deadline={deadline}
Contributed: {amount} lamports, total={raised}
Withdrawn: {amount} lamports
Refunded: {amount} lamports
```

If using Anchor, structured events can also be added later:

```text
CampaignCreated
ContributionMade
CampaignWithdrawn
ContributionRefunded
```

For the MVP, string log requirements must be maintained so behavior is easily verified.

## Initial Testing Plan

Unit/integration tests that must exist during implementation:

1. Create campaign succeeds with a future deadline.
2. Create campaign fails if the deadline has passed.
3. Create campaign fails if the goal is `0`.
4. Contribute succeeds and increases `raised`.
5. Contribute from the same donor accumulates the contribution.
6. Contribute fails if the amount is `0`.
7. Contribute fails after the deadline.
8. Withdraw succeeds if raised >= goal and the deadline has passed.
9. Withdraw fails if the caller is not the creator.
10. Withdraw fails if the goal hasn't been met.
11. Withdraw fails before the deadline.
12. Withdraw fails twice.
13. Refund succeeds if the goal failed and the deadline has passed.
14. Refund fails if the campaign is successful.
15. Refund fails before the deadline.
16. Refund fails twice.
17. Refund only returns the amount belonging to the respective donor.

## QA Specification

This section is used as a quality acceptance checklist to ensure the implementation doesn't deviate from the goal of an escrow crowdfunding platform.

### Success Criteria

- [ ] Accept campaign creation with goal and deadline.
- [ ] Accept contributions and track total raised.
- [ ] Allow withdrawal only if goal reached after deadline.
- [ ] Allow refunds only if goal not reached after deadline.
- [ ] Prevent double withdrawals.
- [ ] Use PDA for vault, not direct transfers to creator.

### Testing Checklist

Happy path successful campaign scenario:

1. Create a campaign with `goal = 1000 SOL`, `deadline = tomorrow`.
2. Contribute `600 SOL`; should succeed and `raised = 600 SOL`.
3. Contribute `500 SOL`; should succeed and `raised = 1100 SOL`.
4. Try withdraw before deadline; should fail.
5. Wait until after deadline.
6. Withdraw should succeed.
7. Try withdraw again; should fail because campaign is already claimed.

Notes:

- SOL values in the checklist are for product-level QA scenarios. In implementation and testing, values must be converted to lamports.
- For automated tests, "wait until after deadline" should ideally be done with a short deadline or by manipulating the local validator/test context if available.

### Common Pitfalls

```text
Do not send donations directly to creator.
Do use PDA vault.

Do not allow withdrawal before deadline.
Do check both goal and time.

Do not forget to mark claimed = true.
Do prevent double withdrawals.

Do not use unwrap() everywhere.
Do handle errors properly.
```

## Resources

Technical references that must be used during implementation:

1. **Program Derived Address (PDA)**

   URL:

   ```text
   https://solanacookbook.com/core-concepts/pdas.html
   ```

   Notes:

   - This Solana Cookbook link currently points to the official Solana documentation on Program-Derived Addresses.
   - Use this reference to understand seeds, bumps, canonical bumps, and the reason PDAs don't have private keys.
   - Directly relevant for vault design:

   ```text
   ["vault", campaign_account.key]
   ```

2. **Cross Program Invocation (CPI)**

   URL:

   ```text
   https://solanacookbook.com/references/programs.html#how-to-do-cross-program-invocation
   ```

   Notes:

   - Use this reference to understand how a program calls another program's instruction.
   - Relevant for transferring SOL via the System Program.
   - Relevant for using `invoke` when a donor sends SOL to the vault.
   - Relevant for using `invoke_signed` when the program transfers SOL from the vault PDA to the creator or donor.

3. **Clock / Current Time**

   Related references are on the Writing Programs page of the Solana Cookbook:

   ```text
   https://solanacookbook.com/references/programs.html#how-to-get-clock-in-a-program
   ```

   Notes:

   - Use `Clock::get()?.unix_timestamp` as the on-chain time source.
   - Do not accept the current time from the client for deadline validation.

## Deliverables

Deliverables that must be available when the project is considered complete:

1. **Rust Program Code**

   Initial status: not available yet.

   Must provide:

   - Solana program source code in Rust.
   - Instruction implementation:
     - `create_campaign`
     - `contribute`
     - `withdraw`
     - `refund`
   - Account/state definitions:
     - `Campaign`
     - `Contribution`
     - Vault PDA
   - Error handling without using unsafe `unwrap()`.
   - Tests that verify success criteria and failure cases.

2. **Deployed To Solana Devnet**

   Initial status: not available yet.

   Must provide:

   - Program successfully built.
   - Program successfully deployed to the Solana Devnet.
   - Target network used:

   ```text
   devnet
   ```

   Minimum proof:

   - Deploy command output.
   - Deployed program address.
   - Devnet explorer link if available.

3. **Program ID**

   Initial status: not available yet.

   Must provide:

   ```text
   Program ID: <to be filled after deploy>
   ```

   Notes:

   - The Program ID must be recorded after a successful deployment.
   - The Program ID must be consistent with the client/test configuration.
   - If using Anchor, `Anchor.toml` and `declare_id!()` must align with the deployed Program ID.

4. **Test Transaction Signatures**

   Initial status: not available yet.

   Must provide:

   - Create campaign transaction signature.
   - Contribute transaction signature.
   - Withdraw transaction signature for a successful campaign.
   - Refund transaction signature for a failed campaign.
   - Signatures for failed transactions are not always available as finalized transactions, but failure cases must still be proven via test output.

   Recording format:

   ```text
   Create Campaign Signature: <signature>
   Contribute Signature: <signature>
   Withdraw Signature: <signature>
   Refund Signature: <signature>
   ```

   If possible, add a Devnet explorer link for each signature:

   ```text
   https://explorer.solana.com/tx/<signature>?cluster=devnet
   ```

## Open Decisions Before Coding

Things that need to be decided before implementation:

1. Final project name: use **VaultRaise** temporarily.
2. Program framework: Anchor is recommended for account ergonomics, PDAs, and tests.
3. Campaign seed: needs an explicit `campaign_id` if a creator can make multiple campaigns.
4. Whether the campaign account will ever be closed or left as an audit trail.
5. Whether the platform will take a fee in advanced versions.
6. Whether the campaign can receive contributions after the goal is reached but before the deadline.

Suggested MVP decisions:

- Use Anchor.
- Use `campaign_id: u64` when creating a campaign.
- Allow contributions to come in before the deadline even if the goal has been reached.
- Do not close campaign accounts in the MVP.
- Do not have platform fees in the MVP.

## MVP Acceptance Criteria

The program is considered to match the context if:

- A creator can create a campaign with a goal and deadline.
- A donor can contribute SOL to the vault PDA, not to the creator.
- The total raised is accurately recorded.
- The creator can only withdraw after the deadline if the goal is reached.
- A donor can only refund after the deadline if the goal failed.
- Funds cannot be taken by the creator before the withdraw conditions are valid.
- A donor cannot refund twice.
- The creator cannot withdraw twice.
- Logs match the primary requirements.
