use anchor_lang::prelude::*;

use crate::errors::VaultRaiseError;
use crate::events::{GovernanceInitialized, GovernanceTransferred};
use crate::state::{Governance, GOVERNANCE_SEED};

pub fn initialize_governance(ctx: Context<InitializeGovernance>) -> Result<()> {
    let governance = &mut ctx.accounts.governance;
    governance.authority = ctx.accounts.authority.key();
    governance.pending_authority = Pubkey::default();
    governance.bump = ctx.bumps.governance;

    emit!(GovernanceInitialized {
        governance: governance.key(),
        authority: governance.authority,
    });

    Ok(())
}

pub fn transfer_governance(ctx: Context<TransferGovernance>, new_authority: Pubkey) -> Result<()> {
    require!(
        new_authority != Pubkey::default(),
        VaultRaiseError::InvalidGovernanceAuthority
    );

    let governance = &mut ctx.accounts.governance;
    let previous_authority = governance.authority;
    governance.authority = new_authority;
    governance.pending_authority = Pubkey::default();

    emit!(GovernanceTransferred {
        governance: governance.key(),
        previous_authority,
        new_authority,
    });

    Ok(())
}

#[derive(Accounts)]
/// Initializes program-level governance authority.
pub struct InitializeGovernance<'info> {
    #[account(
        init,
        payer = authority,
        space = Governance::SPACE,
        seeds = [GOVERNANCE_SEED],
        bump
    )]
    pub governance: Account<'info, Governance>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
/// Updates governance authority. This is intentionally narrow until admin
/// operations are added.
pub struct TransferGovernance<'info> {
    #[account(
        mut,
        has_one = authority @ VaultRaiseError::InvalidGovernanceAuthority
    )]
    pub governance: Account<'info, Governance>,

    pub authority: Signer<'info>,
}
