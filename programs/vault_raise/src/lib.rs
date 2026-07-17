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
