use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod vault_raise {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        msg!("VaultRaise program initialized");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
