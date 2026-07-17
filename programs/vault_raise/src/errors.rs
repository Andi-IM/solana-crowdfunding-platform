use anchor_lang::prelude::*;

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
    #[msg("Metadata URI is too long.")]
    MetadataUriTooLong,
    #[msg("Campaign cannot be closed until it is claimed.")]
    CampaignNotClosable,
    #[msg("Contribution cannot be closed until it is refunded.")]
    ContributionNotClosable,
    #[msg("Native SOL campaign expected.")]
    NativeAssetRequired,
    #[msg("Governance authority cannot be the default public key.")]
    InvalidGovernanceAuthority,
}
