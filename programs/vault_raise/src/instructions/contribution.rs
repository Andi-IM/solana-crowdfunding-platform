use anchor_lang::prelude::*;

use crate::errors::VaultRaiseError;
use crate::events::{CampaignContributed, ContributionClosed, ContributionRefunded};
use crate::state::{
    vault_signer_seeds, Campaign, CampaignStatus, Contribution, CONTRIBUTION_SEED, VAULT_SEED,
};

pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;

    require!(amount > 0, VaultRaiseError::InvalidContributionAmount);

    let campaign = &mut ctx.accounts.campaign;

    require!(
        campaign.asset.is_native_sol(),
        VaultRaiseError::NativeAssetRequired
    );
    require!(
        current_time < campaign.deadline,
        VaultRaiseError::CampaignEnded
    );
    require!(!campaign.claimed, VaultRaiseError::AlreadyClaimed);
    require!(
        campaign.status == CampaignStatus::Active,
        VaultRaiseError::CampaignNotActive
    );

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

pub fn refund(ctx: Context<Refund>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let campaign = &ctx.accounts.campaign;
    let contribution = &mut ctx.accounts.contribution;

    require!(
        campaign.asset.is_native_sol(),
        VaultRaiseError::NativeAssetRequired
    );
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

pub fn close_refunded_contribution(ctx: Context<CloseRefundedContribution>) -> Result<()> {
    require!(
        ctx.accounts.contribution.refunded,
        VaultRaiseError::ContributionNotClosable
    );

    emit!(ContributionClosed {
        campaign: ctx.accounts.campaign.key(),
        donor: ctx.accounts.donor.key(),
    });

    Ok(())
}

#[derive(Accounts)]
/// Accounts required for a donor contribution into a native-SOL campaign vault.
pub struct Contribute<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    #[account(
        init_if_needed,
        payer = donor,
        space = Contribution::SPACE,
        seeds = [CONTRIBUTION_SEED, campaign.key().as_ref(), donor.key().as_ref()],
        bump
    )]
    pub contribution: Account<'info, Contribution>,

    /// CHECK: Vault PDA to hold native SOL campaign funds.
    #[account(
        mut,
        seeds = [VAULT_SEED, campaign.key().as_ref()],
        bump = campaign.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub donor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
/// Accounts required for a donor refund from a failed native-SOL campaign.
pub struct Refund<'info> {
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        has_one = campaign,
        has_one = donor,
    )]
    pub contribution: Account<'info, Contribution>,

    /// CHECK: Vault PDA to hold native SOL campaign funds.
    #[account(
        mut,
        seeds = [VAULT_SEED, campaign.key().as_ref()],
        bump = campaign.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub donor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
/// Closes refunded contribution state and returns rent to the donor.
pub struct CloseRefundedContribution<'info> {
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        close = donor,
        has_one = campaign,
        has_one = donor,
    )]
    pub contribution: Account<'info, Contribution>,

    #[account(mut)]
    pub donor: Signer<'info>,
}
