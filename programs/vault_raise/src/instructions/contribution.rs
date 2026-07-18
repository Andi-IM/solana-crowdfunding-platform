use anchor_lang::prelude::*;

use crate::errors::VaultRaiseError;
use crate::events::{CampaignContributed, ContributionClosed, ContributionRefunded};
use crate::instructions::native_sol::{transfer_from_signer, transfer_from_vault};
use crate::state::{Campaign, Contribution, CONTRIBUTION_SEED, VAULT_SEED};

pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;

    require!(amount > 0, VaultRaiseError::InvalidContributionAmount);

    let campaign = &mut ctx.accounts.campaign;
    campaign.ensure_contributable(current_time)?;

    transfer_from_signer(
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.donor.to_account_info(),
        ctx.accounts.vault.to_account_info(),
        amount,
    )?;

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

    msg!(
        "Contributed: {} lamports, total={}",
        amount,
        campaign.raised
    );

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
    let campaign = &mut ctx.accounts.campaign;
    let contribution = &mut ctx.accounts.contribution;

    campaign.ensure_refundable(current_time)?;
    require!(!contribution.refunded, VaultRaiseError::AlreadyRefunded);
    require!(
        contribution.amount > 0,
        VaultRaiseError::InvalidContributionAmount
    );

    let amount = contribution.amount;
    let campaign_key = campaign.key();
    transfer_from_vault(
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.donor.to_account_info(),
        &campaign_key,
        campaign.vault_bump,
        amount,
    )?;

    contribution.refunded = true;
    contribution.amount = 0;
    campaign.refunded = campaign
        .refunded
        .checked_add(amount)
        .ok_or(VaultRaiseError::ArithmeticOverflow)?;

    msg!("Refunded: {} lamports", amount);

    emit!(ContributionRefunded {
        campaign: campaign.key(),
        donor: ctx.accounts.donor.key(),
        amount,
    });

    Ok(())
}

pub fn close_refunded_contribution(ctx: Context<CloseRefundedContribution>) -> Result<()> {
    require!(
        ctx.accounts.contribution.refunded || ctx.accounts.campaign.claimed,
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
    #[account(mut)]
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
