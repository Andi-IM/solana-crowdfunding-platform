use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod vault_raise {
    use super::*;

    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        _campaign_id: u64,
        goal: u64,
        deadline: i64,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        require!(goal > 0, VaultRaiseError::InvalidGoal);
        require!(deadline > current_time, VaultRaiseError::InvalidDeadline);

        let campaign = &mut ctx.accounts.campaign;
        campaign.creator = ctx.accounts.creator.key();
        campaign.goal = goal;
        campaign.raised = 0;
        campaign.deadline = deadline;
        campaign.claimed = false;
        campaign.bump = ctx.bumps.campaign;
        campaign.vault_bump = ctx.bumps.vault;

        // INTERNAL DOCUMENTATION:
        // The creator must not receive donations directly. All contributions
        // are stored in the program-controlled vault PDA.

        msg!("Campaign created: goal={}, deadline={}", goal, deadline);

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(_campaign_id: u64)]
pub struct CreateCampaign<'info> {
    #[account(
        init,
        payer = creator,
        space = Campaign::SPACE,
        seeds = [b"campaign", creator.key().as_ref(), &_campaign_id.to_le_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    /// CHECK: Vault PDA to hold campaign funds.
    /// It must not be the creator's direct account.
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Campaign {
    pub creator: Pubkey,
    pub goal: u64,
    pub raised: u64,
    pub deadline: i64,
    pub claimed: bool,
    pub bump: u8,
    pub vault_bump: u8,
}

impl Campaign {
    pub const SPACE: usize = 8 + Self::INIT_SPACE;
}

#[account]
#[derive(InitSpace)]
pub struct Contribution {
    pub campaign: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
    pub refunded: bool,
    pub bump: u8,
}

impl Contribution {
    pub const SPACE: usize = 8 + Self::INIT_SPACE;
}

#[error_code]
pub enum VaultRaiseError {
    #[msg("Goal must be greater than zero.")]
    InvalidGoal,
    #[msg("Deadline must be in the future.")]
    InvalidDeadline,
    #[msg("Campaign has already ended.")]
    CampaignEnded,
    #[msg("Campaign has not ended yet.")]
    CampaignNotEnded,
    #[msg("Campaign has not reached its goal.")]
    CampaignNotSuccessful,
    #[msg("Campaign reached its goal and is not eligible for refunds.")]
    CampaignNotFailed,
    #[msg("Only the campaign creator can perform this action.")]
    UnauthorizedCreator,
    #[msg("Campaign funds have already been claimed.")]
    AlreadyClaimed,
    #[msg("Contribution has already been refunded.")]
    AlreadyRefunded,
    #[msg("Contribution amount must be greater than zero.")]
    InvalidContributionAmount,
    #[msg("Arithmetic overflow.")]
    ArithmeticOverflow,
    #[msg("Vault balance is insufficient.")]
    InsufficientVaultBalance,
}
