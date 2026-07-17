# VaultRaise - Jira Task Breakdown

This document contains a Jira-style task breakdown for the development of the MVP Solana Crowdfunding Platform. Project working name: **VaultRaise**.

## EPIC-001 - Project Foundation

### VR-001 - Initialize Solana/Anchor Project Structure

**Type:** Task  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Set up the initial Solana project structure using Anchor so that the program, tests, and deployment can be managed consistently.

**Acceptance Criteria:**

- Anchor project is successfully created.
- Rust program folder structure is available.
- `Anchor.toml` is available.
- The project can run an initial build.
- Local AI files remain untracked by git.

**Dependencies:** None

### VR-002 - Define Program Accounts And Error Types

**Type:** Task  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Implement the main account structures for campaign, contribution, and custom errors.

**Acceptance Criteria:**

- `Campaign` account has `creator`, `goal`, `raised`, `deadline`, and `claimed`.
- `Contribution` account stores the campaign, donor, amount, and refunded status.
- Custom errors are available for invalid deadline, unauthorized creator, already claimed, already refunded, arithmetic overflow, and invalid amount.
- No `unwrap()` is used for flows that can fail.

**Dependencies:** VR-001

## EPIC-002 - Campaign Lifecycle

### VR-003 - Implement Create Campaign Instruction

**Type:** Story  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
As a creator, I want to create a campaign with a goal and a deadline so that donors can contribute to it.

**Acceptance Criteria:**

- The instruction accepts `goal: u64` and `deadline: i64`.
- The program rejects `goal = 0`.
- The program rejects a deadline that is not in the future.
- The campaign stores creator, goal, deadline, raised `0`, and claimed `false`.
- The program logs: `Campaign created: goal={goal}, deadline={deadline}`.

**Dependencies:** VR-002

### VR-004 - Implement PDA Vault Derivation

**Type:** Task  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Implement a vault PDA so that all campaign funds are locked by the program, instead of being sent directly to the creator.

**Acceptance Criteria:**

- Vault PDA is derived with the seed `["vault", campaign.key()]`.
- The bump is stored or can be verified securely.
- The program validates the provided vault account against the seed.
- Internal documentation explains that the creator must not receive donations directly.

**Dependencies:** VR-003

### VR-005 - Implement Contribute Instruction

**Type:** Story  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 2 days  

**Description:**  
As a donor, I want to send SOL to the campaign vault so that my funds are locked until the campaign succeeds or fails.

**Acceptance Criteria:**

- The instruction accepts `amount: u64`.
- The program rejects `amount = 0`.
- The program rejects contributions after the deadline.
- The program transfers SOL from the donor to the vault PDA.
- The program increases `campaign.raised` using checked arithmetic.
- The program creates or updates the `Contribution` account.
- The program logs: `Contributed: {amount} lamports, total={raised}`.

**Dependencies:** VR-004

### VR-006 - Implement Withdraw Instruction

**Type:** Story  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 2 days  

**Description:**  
As a creator, I want to withdraw funds from the vault if the campaign reaches its goal after the deadline.

**Acceptance Criteria:**

- Withdraw only succeeds if `raised >= goal`.
- Withdraw only succeeds if current time `>= deadline`.
- Withdraw only succeeds if the caller is the creator.
- Withdraw fails if the campaign is already claimed.
- The program transfers SOL from the vault PDA to the creator using the signed PDA flow.
- The program marks `claimed = true`.
- The program logs: `Withdrawn: {amount} lamports`.

**Dependencies:** VR-005

### VR-007 - Implement Refund Instruction

**Type:** Story  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 2 days  

**Description:**  
As a donor, I want to get a refund if the campaign fails to reach its goal after the deadline.

**Acceptance Criteria:**

- Refund only succeeds if `raised < goal`.
- Refund only succeeds if current time `>= deadline`.
- Refund only succeeds for a donor who has a contribution.
- Refund fails if the contribution is already refunded.
- The program transfers SOL from the vault PDA to the donor using the signed PDA flow.
- The program marks the contribution as refunded.
- The program logs: `Refunded: {amount} lamports`.

**Dependencies:** VR-005

## EPIC-003 - Testing And QA

### VR-008 - Write Unit And Integration Tests For Campaign Creation

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Write tests to ensure that valid and invalid campaign creations behave according to the specifications.

**Acceptance Criteria:**

- Campaign creation test succeeds with a future deadline.
- Campaign creation test fails if the deadline has passed.
- Campaign creation test fails if the goal is `0`.
- The initial state of the campaign is validated.

**Dependencies:** VR-003

### VR-009 - Write Tests For Contribution Flow

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Write tests for donor contributions and the accumulation of total raised funds.

**Acceptance Criteria:**

- Contribute `600 SOL` equivalent in lamports succeeds.
- Additional contribute `500 SOL` equivalent in lamports succeeds.
- `raised` becomes a total of `1100 SOL` equivalent in lamports.
- The donor's contribution is recorded correctly.
- Contribute after the deadline fails.
- Contribute amount `0` fails.

**Dependencies:** VR-005

### VR-010 - Write Tests For Withdraw Flow

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Write withdraw tests for successful campaigns and failure cases.

**Acceptance Criteria:**

- Withdraw before the deadline fails.
- Withdraw after the deadline and when the goal is reached succeeds.
- Withdraw by a non-creator fails.
- A second withdraw fails because it's already claimed.
- The creator's balance increases according to the vault's funds.

**Dependencies:** VR-006

### VR-011 - Write Tests For Refund Flow

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Write refund tests for failed campaigns and failure cases.

**Acceptance Criteria:**

- Refund before the deadline fails.
- Refund after the deadline and when the goal is not reached succeeds.
- Refund for a successful campaign fails.
- A second refund fails because it's already refunded.
- Refund only returns the amount belonging to the respective donor.

**Dependencies:** VR-007

### VR-012 - Run QA Checklist And Capture Evidence

**Type:** Task  
**Priority:** High  
**Assignee:** QA Engineer  
**Estimate:** 1 day  

**Description:**  
Run the existing QA checklist in the project context and record the results as evidence prior to deployment.

**Acceptance Criteria:**

- All QA success criteria are marked pass/fail.
- Successful campaign scenarios are executed end-to-end.
- Failed campaign refund scenarios are executed end-to-end.
- All critical failure cases have test evidence.
- No direct transfers to the creator occur during a contribution.

**Dependencies:** VR-008, VR-009, VR-010, VR-011

## EPIC-004 - Devnet Deployment

### VR-013 - Prepare Devnet Wallet And Configuration

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 0.5 day  

**Description:**  
Prepare the wallet and Devnet cluster configuration for program deployment.

**Acceptance Criteria:**

- Solana CLI target cluster is Devnet.
- The deployer wallet is available.
- The wallet has sufficient Devnet SOL.
- Anchor configuration points to Devnet.

**Dependencies:** VR-012

### VR-014 - Deploy Program To Solana Devnet

**Type:** Task  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Deploy the Rust program to the Solana Devnet and record the Program ID.

**Acceptance Criteria:**

- The program is successfully built for deployment.
- The program is successfully deployed to Devnet.
- The Program ID is recorded in the deliverables documentation.
- If using Anchor, `declare_id!()` and `Anchor.toml` align with the Program ID.
- A Devnet explorer link is provided if possible.

**Dependencies:** VR-013

### VR-015 - Execute Devnet Test Transactions

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Execute Devnet transactions to prove campaign creation, contribution, withdrawal, and refund.

**Acceptance Criteria:**

- Create campaign transaction signature is recorded.
- Contribute transaction signature is recorded.
- Successful campaign withdraw transaction signature is recorded.
- Failed campaign refund transaction signature is recorded.
- Devnet explorer links for each signature are recorded if available.

**Dependencies:** VR-014

## EPIC-005 - Documentation And Handoff

### VR-016 - Update Project Context With Implementation Decisions

**Type:** Task  
**Priority:** Medium  
**Assignee:** Technical Writer / Blockchain Engineer  
**Estimate:** 0.5 day  

**Description:**  
Update the context document with final decisions that emerged during implementation.

**Acceptance Criteria:**

- The final project name or working name is confirmed.
- The final campaign seed is documented.
- The Program ID is documented.
- Devnet deployment evidence is documented.
- Test transaction signatures are documented.

**Dependencies:** VR-015

### VR-017 - Prepare Developer Handoff Notes

**Type:** Task  
**Priority:** Medium  
**Assignee:** Technical Writer / Project Manager  
**Estimate:** 0.5 day  

**Description:**  
Prepare handoff notes so the next programmer can run builds, tests, and deployments without losing context.

**Acceptance Criteria:**

- Local environment setup instructions are available.
- Build instructions are available.
- Test instructions are available.
- Devnet deployment instructions are available.
- MVP known limitations are documented.

**Dependencies:** VR-016

## Delivery Milestones

1. **Milestone 1 - Foundation Ready**
   - VR-001
   - VR-002

2. **Milestone 2 - Core Program Complete**
   - VR-003
   - VR-004
   - VR-005
   - VR-006
   - VR-007

3. **Milestone 3 - Test Coverage Complete**
   - VR-008
   - VR-009
   - VR-010
   - VR-011
   - VR-012

4. **Milestone 4 - Devnet Delivered**
   - VR-013
   - VR-014
   - VR-015

5. **Milestone 5 - Handoff Complete**
   - VR-016
   - VR-017

## Definition Of Done

The MVP project is considered complete if:

- The Rust program code is available.
- All core instructions are complete: create campaign, contribute, withdraw, refund.
- Core tests and failure cases pass.
- The program is successfully deployed to the Solana Devnet.
- The Program ID is recorded.
- Test transaction signatures are recorded.
- No contribution funds are sent directly to the creator.
- A PDA vault is used to escrow funds.
- Context and handoff documentation are updated.
