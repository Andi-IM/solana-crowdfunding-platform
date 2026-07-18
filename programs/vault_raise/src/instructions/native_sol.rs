use anchor_lang::prelude::*;

use crate::state::vault_signer_seeds;

pub fn transfer_from_signer<'info>(
    system_program: AccountInfo<'info>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let cpi_context = CpiContext::new(
        system_program,
        anchor_lang::system_program::Transfer { from, to },
    );

    anchor_lang::system_program::transfer(cpi_context, amount)
}

pub fn transfer_from_vault<'info>(
    system_program: AccountInfo<'info>,
    vault: AccountInfo<'info>,
    recipient: AccountInfo<'info>,
    campaign: &Pubkey,
    vault_bump: u8,
    amount: u64,
) -> Result<()> {
    let vault_bump = [vault_bump];
    let seeds = vault_signer_seeds(campaign, &vault_bump);
    let signer_seeds = &[&seeds[..]];
    let cpi_context = CpiContext::new_with_signer(
        system_program,
        anchor_lang::system_program::Transfer {
            from: vault,
            to: recipient,
        },
        signer_seeds,
    );

    anchor_lang::system_program::transfer(cpi_context, amount)
}
