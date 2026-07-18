#![allow(ambiguous_glob_reexports)]

use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use events::*;
pub use instructions::*;
pub use state::*;

declare_id!("GeYMy79EJmUs8japokaVcadb2RRs6vv7c4xYE2fbjkQW");

#[program]
pub mod vault_raise {
    use super::*;

    /// Creates a new native-SOL campaign.
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        campaign_id: u64,
        goal: u64,
        deadline: i64,
    ) -> Result<()> {
        instructions::create_campaign(ctx, campaign_id, goal, deadline)
    }

    /// Updates campaign metadata with an explicit realloc boundary.
    pub fn update_campaign_metadata(
        ctx: Context<UpdateCampaignMetadata>,
        metadata_uri: String,
    ) -> Result<()> {
        instructions::update_campaign_metadata(ctx, metadata_uri)
    }

    /// Contributes native SOL to an active campaign vault.
    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        instructions::contribute(ctx, amount)
    }

    /// Withdraws all vault SOL to the campaign creator after a successful campaign.
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw(ctx)
    }

    /// Sweeps direct-transfer surplus from a failed campaign vault while preserving tracked refunds.
    pub fn sweep_failed_vault_surplus(ctx: Context<SweepFailedVaultSurplus>) -> Result<()> {
        instructions::sweep_failed_vault_surplus(ctx)
    }

    /// Refunds a donor's recorded contribution after a failed campaign.
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund(ctx)
    }

    /// Closes a claimed campaign account and returns rent to the creator.
    pub fn close_campaign(ctx: Context<CloseCampaign>) -> Result<()> {
        instructions::close_campaign(ctx)
    }

    /// Closes a refunded contribution account and returns rent to the donor.
    pub fn close_refunded_contribution(ctx: Context<CloseRefundedContribution>) -> Result<()> {
        instructions::close_refunded_contribution(ctx)
    }

    /// Initializes program governance authority.
    pub fn initialize_governance(ctx: Context<InitializeGovernance>) -> Result<()> {
        instructions::initialize_governance(ctx)
    }

    /// Transfers governance authority to a new authority.
    pub fn transfer_governance(
        ctx: Context<TransferGovernance>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::transfer_governance(ctx, new_authority)
    }
}
