use anchor_lang::prelude::*;

use crate::errors::VaultRaiseError;
use crate::events::{
    CampaignClosed, CampaignCreated, CampaignMetadataUpdated, CampaignWithdrawn, VaultSurplusSwept,
};
use crate::instructions::native_sol::{transfer_from_signer, transfer_from_vault};
use crate::state::{
    Campaign, CampaignStatus, FundingAsset, CAMPAIGN_SEED, MAX_METADATA_URI_LEN, VAULT_SEED,
};

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
    campaign.refunded = 0;
    campaign.deadline = deadline;
    campaign.claimed = false;
    campaign.status = CampaignStatus::Active;
    campaign.asset = FundingAsset::NativeSol;
    campaign.metadata_uri_len = 0;
    campaign.bump = ctx.bumps.campaign;
    campaign.vault_bump = ctx.bumps.vault;
    campaign.metadata_uri = String::new();

    let rent_exempt_minimum = Rent::get()?.minimum_balance(0);
    let current_vault_lamports = ctx.accounts.vault.lamports();
    if current_vault_lamports < rent_exempt_minimum {
        transfer_from_signer(
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.creator.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            rent_exempt_minimum - current_vault_lamports,
        )?;
    }

    msg!("Campaign created: goal={}, deadline={}", goal, deadline);

    emit!(CampaignCreated {
        campaign: campaign.key(),
        creator: campaign.creator,
        campaign_id,
        goal,
        deadline,
        vault: ctx.accounts.vault.key(),
        asset: campaign.asset.mint(),
    });

    Ok(())
}

pub fn update_campaign_metadata(
    ctx: Context<UpdateCampaignMetadata>,
    metadata_uri: String,
) -> Result<()> {
    require!(
        metadata_uri.len() <= MAX_METADATA_URI_LEN,
        VaultRaiseError::MetadataUriTooLong
    );

    let campaign = &mut ctx.accounts.campaign;
    campaign.metadata_uri_len =
        u16::try_from(metadata_uri.len()).map_err(|_| VaultRaiseError::MetadataUriTooLong)?;
    campaign.metadata_uri = metadata_uri;

    emit!(CampaignMetadataUpdated {
        campaign: campaign.key(),
        creator: ctx.accounts.creator.key(),
        metadata_uri: campaign.metadata_uri.clone(),
    });

    Ok(())
}

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let campaign = &mut ctx.accounts.campaign;

    campaign.ensure_withdrawable(current_time)?;

    let amount = ctx.accounts.vault.lamports();
    let campaign_key = campaign.key();
    transfer_from_vault(
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.creator.to_account_info(),
        &campaign_key,
        campaign.vault_bump,
        amount,
    )?;

    campaign.mark_claimed();

    msg!("Withdrawn: {} lamports", amount);

    emit!(CampaignWithdrawn {
        campaign: campaign.key(),
        creator: ctx.accounts.creator.key(),
        amount,
    });

    Ok(())
}

pub fn sweep_failed_vault_surplus(ctx: Context<SweepFailedVaultSurplus>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let campaign = &ctx.accounts.campaign;

    campaign.ensure_refundable(current_time)?;

    let rent_exempt_minimum = Rent::get()?.minimum_balance(0);
    let reserved_for_refunds = campaign.tracked_outstanding()?;
    let reserved_balance = reserved_for_refunds
        .checked_add(rent_exempt_minimum)
        .ok_or(VaultRaiseError::ArithmeticOverflow)?;
    let vault_balance = ctx.accounts.vault.lamports();

    require!(
        vault_balance > reserved_balance,
        VaultRaiseError::NoVaultSurplus
    );

    let amount = vault_balance
        .checked_sub(reserved_balance)
        .ok_or(VaultRaiseError::ArithmeticOverflow)?;
    let campaign_key = campaign.key();
    transfer_from_vault(
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.creator.to_account_info(),
        &campaign_key,
        campaign.vault_bump,
        amount,
    )?;

    emit!(VaultSurplusSwept {
        campaign: campaign.key(),
        creator: ctx.accounts.creator.key(),
        amount,
    });

    Ok(())
}

pub fn close_campaign(ctx: Context<CloseCampaign>) -> Result<()> {
    require!(
        ctx.accounts.campaign.status == CampaignStatus::Claimed,
        VaultRaiseError::CampaignNotClosable
    );

    emit!(CampaignClosed {
        campaign: ctx.accounts.campaign.key(),
        creator: ctx.accounts.creator.key(),
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(campaign_id: u64)]
/// Accounts required to initialize a native-SOL campaign and validate its vault PDA.
pub struct CreateCampaign<'info> {
    #[account(
        init,
        payer = creator,
        space = Campaign::SPACE,
        seeds = [CAMPAIGN_SEED, creator.key().as_ref(), &campaign_id.to_le_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    /// CHECK: Vault PDA to hold native SOL campaign funds.
    #[account(
        mut,
        seeds = [VAULT_SEED, campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(metadata_uri: String)]
/// Reallocates campaign metadata within an explicit maximum size.
pub struct UpdateCampaignMetadata<'info> {
    #[account(
        mut,
        has_one = creator @ VaultRaiseError::UnauthorizedCreator,
        realloc = Campaign::realloc_space(metadata_uri.len()),
        realloc::payer = creator,
        realloc::zero = false
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
/// Accounts required for a creator withdrawal from a successful native-SOL campaign.
pub struct Withdraw<'info> {
    #[account(
        mut,
        has_one = creator @ VaultRaiseError::UnauthorizedCreator
    )]
    pub campaign: Account<'info, Campaign>,

    /// CHECK: Vault PDA to hold native SOL campaign funds.
    #[account(
        mut,
        seeds = [VAULT_SEED, campaign.key().as_ref()],
        bump = campaign.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
/// Sweeps untracked lamports sent directly to a failed native-SOL vault.
pub struct SweepFailedVaultSurplus<'info> {
    #[account(
        has_one = creator @ VaultRaiseError::UnauthorizedCreator
    )]
    pub campaign: Account<'info, Campaign>,

    /// CHECK: Vault PDA to hold native SOL campaign funds.
    #[account(
        mut,
        seeds = [VAULT_SEED, campaign.key().as_ref()],
        bump = campaign.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
/// Closes claimed campaign state and returns rent to the creator.
pub struct CloseCampaign<'info> {
    #[account(
        mut,
        close = creator,
        has_one = creator @ VaultRaiseError::UnauthorizedCreator
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(mut)]
    pub creator: Signer<'info>,
}
