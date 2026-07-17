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

    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        require!(amount > 0, VaultRaiseError::InvalidContributionAmount);

        let campaign = &mut ctx.accounts.campaign;

        require!(current_time < campaign.deadline, VaultRaiseError::CampaignEnded);
        require!(!campaign.claimed, VaultRaiseError::AlreadyClaimed);

        // Transfer SOL from donor to vault
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.donor.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, amount)?;

        // Update campaign raised amount
        campaign.raised = campaign
            .raised
            .checked_add(amount)
            .ok_or(VaultRaiseError::ArithmeticOverflow)?;

        // Create or update contribution account
        let contribution = &mut ctx.accounts.contribution;
        if contribution.amount == 0 && !contribution.refunded {
            contribution.campaign = campaign.key();
            contribution.donor = ctx.accounts.donor.key();
            contribution.amount = amount;
            contribution.refunded = false;
            contribution.bump = ctx.bumps.contribution;
        } else {
            contribution.amount = contribution
                .amount
                .checked_add(amount)
                .ok_or(VaultRaiseError::ArithmeticOverflow)?;
        }

        msg!("Contributed: {} lamports, total={}", amount, campaign.raised);

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let campaign = &mut ctx.accounts.campaign;

        require!(campaign.raised >= campaign.goal, VaultRaiseError::CampaignNotSuccessful);
        require!(current_time >= campaign.deadline, VaultRaiseError::CampaignNotEnded);
        require!(!campaign.claimed, VaultRaiseError::AlreadyClaimed);

        let amount = ctx.accounts.vault.lamports();
        
        let campaign_key = campaign.key();
        let vault_bump = campaign.vault_bump;
        let seeds = &[
            b"vault".as_ref(),
            campaign_key.as_ref(),
            &[vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.creator.to_account_info(),
            },
            signer_seeds,
        );

        anchor_lang::system_program::transfer(cpi_context, amount)?;

        campaign.claimed = true;

        msg!("Withdrawn: {} lamports", amount);

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

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    #[account(
        init_if_needed,
        payer = donor,
        space = Contribution::SPACE,
        seeds = [b"contribution", campaign.key().as_ref(), donor.key().as_ref()],
        bump
    )]
    pub contribution: Account<'info, Contribution>,

    /// CHECK: Vault PDA to hold campaign funds.
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump = campaign.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub donor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        has_one = creator @ VaultRaiseError::UnauthorizedCreator
    )]
    pub campaign: Account<'info, Campaign>,

    /// CHECK: Vault PDA to hold campaign funds.
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump = campaign.vault_bump
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
