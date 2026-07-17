use anchor_lang::prelude::*;

declare_id!("GeYMy79EJmUs8japokaVcadb2RRs6vv7c4xYE2fbjkQW");

#[program]
pub mod vault_raise {
    use super::*;

    /// Creates a new campaign account and records its vault PDA bump.
    ///
    /// `campaign_id` is part of the campaign PDA seed so one creator can own
    /// multiple campaigns. `goal` is denominated in lamports and must be
    /// greater than zero. `deadline` is a Unix timestamp and must be in the
    /// future according to Solana's on-chain `Clock` sysvar.
    ///
    /// Side effects:
    /// - Initializes the `Campaign` account.
    /// - Validates the associated vault PDA.
    /// - Emits a creation log for off-chain verification.
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        campaign_id: u64,
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
        campaign.status = CampaignStatus::Active;
        campaign.bump = ctx.bumps.campaign;
        campaign.vault_bump = ctx.bumps.vault;

        // All contributions must go through the vault PDA so the creator cannot
        // receive funds before the campaign success conditions are satisfied.

        emit!(CampaignCreated {
            campaign: campaign.key(),
            creator: campaign.creator,
            campaign_id,
            goal,
            deadline,
            vault: ctx.accounts.vault.key(),
        });

        Ok(())
    }

    /// Contributes native SOL to an active campaign vault.
    ///
    /// `amount` is denominated in lamports and must be greater than zero.
    /// Contributions are only accepted before the campaign deadline and before
    /// the campaign has been claimed. The donor's contribution account is
    /// created on first contribution and accumulated on later contributions.
    ///
    /// Side effects:
    /// - Transfers SOL from donor to the vault PDA through the System Program.
    /// - Increments `campaign.raised` with checked arithmetic.
    /// - Creates or updates the donor's `Contribution` account.
    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        require!(amount > 0, VaultRaiseError::InvalidContributionAmount);

        let campaign = &mut ctx.accounts.campaign;

        require!(
            current_time < campaign.deadline,
            VaultRaiseError::CampaignEnded
        );
        require!(!campaign.claimed, VaultRaiseError::AlreadyClaimed);
        require!(
            campaign.status == CampaignStatus::Active,
            VaultRaiseError::CampaignNotActive
        );

        // Use the System Program for the lamport transfer; this preserves the
        // campaign escrow invariant that funds are held by the vault PDA.
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.donor.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, amount)?;

        campaign.raised = campaign
            .raised
            .checked_add(amount)
            .ok_or(VaultRaiseError::ArithmeticOverflow)?;

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

        emit!(CampaignContributed {
            campaign: campaign.key(),
            donor: ctx.accounts.donor.key(),
            amount,
            total_raised: campaign.raised,
        });

        Ok(())
    }

    /// Withdraws all vault SOL to the campaign creator after a successful campaign.
    ///
    /// The campaign must have reached its goal, the deadline must have passed,
    /// and the signer must match `campaign.creator`.
    ///
    /// Side effects:
    /// - Transfers all vault lamports to the creator using the vault PDA signer.
    /// - Marks the campaign as claimed to prevent a second withdrawal.
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let campaign = &mut ctx.accounts.campaign;

        require!(
            campaign.raised >= campaign.goal,
            VaultRaiseError::CampaignNotSuccessful
        );
        require!(
            current_time >= campaign.deadline,
            VaultRaiseError::CampaignNotEnded
        );
        require!(!campaign.claimed, VaultRaiseError::AlreadyClaimed);

        let amount = ctx.accounts.vault.lamports();

        let campaign_key = campaign.key();
        let vault_bump = [campaign.vault_bump];
        let seeds = vault_signer_seeds(&campaign_key, &vault_bump);
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
        campaign.status = CampaignStatus::Claimed;

        emit!(CampaignWithdrawn {
            campaign: campaign.key(),
            creator: ctx.accounts.creator.key(),
            amount,
        });

        Ok(())
    }

    /// Refunds a donor's recorded contribution after a failed campaign.
    ///
    /// The campaign must be past its deadline and below its funding goal. The
    /// contribution account must belong to the donor and campaign enforced by
    /// the account constraints.
    ///
    /// Side effects:
    /// - Transfers the contribution amount from the vault PDA back to the donor.
    /// - Marks the contribution as refunded to prevent double refunds.
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let campaign = &ctx.accounts.campaign;
        let contribution = &mut ctx.accounts.contribution;

        require!(
            campaign.raised < campaign.goal,
            VaultRaiseError::CampaignNotFailed
        );
        require!(
            current_time >= campaign.deadline,
            VaultRaiseError::CampaignNotEnded
        );
        require!(
            campaign.status == CampaignStatus::Active,
            VaultRaiseError::CampaignNotActive
        );
        require!(!contribution.refunded, VaultRaiseError::AlreadyRefunded);
        require!(
            contribution.amount > 0,
            VaultRaiseError::InvalidContributionAmount
        );

        let amount = contribution.amount;

        let campaign_key = campaign.key();
        let vault_bump = [campaign.vault_bump];
        let seeds = vault_signer_seeds(&campaign_key, &vault_bump);
        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.donor.to_account_info(),
            },
            signer_seeds,
        );

        anchor_lang::system_program::transfer(cpi_context, amount)?;

        contribution.refunded = true;

        emit!(ContributionRefunded {
            campaign: campaign.key(),
            donor: ctx.accounts.donor.key(),
            amount,
        });

        Ok(())
    }
}

fn vault_signer_seeds<'a>(campaign: &'a Pubkey, vault_bump: &'a [u8; 1]) -> [&'a [u8]; 3] {
    [b"vault".as_ref(), campaign.as_ref(), vault_bump]
}

#[derive(Accounts)]
#[instruction(campaign_id: u64)]
/// Accounts required to initialize a campaign and validate its vault PDA.
pub struct CreateCampaign<'info> {
    #[account(
        init,
        payer = creator,
        space = Campaign::SPACE,
        seeds = [b"campaign", creator.key().as_ref(), &campaign_id.to_le_bytes()],
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
/// Accounts required for a donor contribution into a campaign vault.
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
/// Accounts required for a creator withdrawal from a successful campaign.
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

#[derive(Accounts)]
/// Accounts required for a donor refund from a failed campaign.
pub struct Refund<'info> {
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        has_one = campaign,
        has_one = donor,
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

#[account]
#[derive(InitSpace)]
/// Persistent campaign state.
pub struct Campaign {
    pub creator: Pubkey,
    pub goal: u64,
    pub raised: u64,
    pub deadline: i64,
    pub claimed: bool,
    pub status: CampaignStatus,
    pub bump: u8,
    pub vault_bump: u8,
}

impl Campaign {
    pub const SPACE: usize = 8 + Self::INIT_SPACE;
}

#[account]
#[derive(InitSpace)]
/// Per-donor contribution state used to calculate and gate refunds.
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
/// Explicit lifecycle marker for campaign state transitions.
pub enum CampaignStatus {
    Active,
    Claimed,
}

#[event]
pub struct CampaignCreated {
    pub campaign: Pubkey,
    pub creator: Pubkey,
    pub campaign_id: u64,
    pub goal: u64,
    pub deadline: i64,
    pub vault: Pubkey,
}

#[event]
pub struct CampaignContributed {
    pub campaign: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
    pub total_raised: u64,
}

#[event]
pub struct CampaignWithdrawn {
    pub campaign: Pubkey,
    pub creator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ContributionRefunded {
    pub campaign: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
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
    #[msg("Campaign is not active.")]
    CampaignNotActive,
}
