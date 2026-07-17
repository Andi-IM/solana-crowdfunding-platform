use anchor_lang::prelude::*;

pub const GOVERNANCE_SEED: &[u8] = b"governance";
pub const CAMPAIGN_SEED: &[u8] = b"campaign";
pub const VAULT_SEED: &[u8] = b"vault";
pub const CONTRIBUTION_SEED: &[u8] = b"contribution";
pub const MAX_METADATA_URI_LEN: usize = 200;

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
    pub asset: FundingAsset,
    pub metadata_uri_len: u16,
    pub bump: u8,
    pub vault_bump: u8,
    #[max_len(MAX_METADATA_URI_LEN)]
    pub metadata_uri: String,
}

impl Campaign {
    pub const SPACE: usize = 8 + Self::INIT_SPACE;

    pub fn realloc_space(metadata_uri_len: usize) -> usize {
        8 + 32 + 8 + 8 + 8 + 1 + 1 + FundingAsset::INIT_SPACE + 2 + 1 + 1 + 4 + metadata_uri_len
    }
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

#[account]
#[derive(InitSpace)]
/// Program-level governance configuration for future administrative controls.
pub struct Governance {
    pub authority: Pubkey,
    pub pending_authority: Pubkey,
    pub bump: u8,
}

impl Governance {
    pub const SPACE: usize = 8 + Self::INIT_SPACE;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
/// Explicit lifecycle marker for campaign state transitions.
pub enum CampaignStatus {
    Active,
    Claimed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
/// Funding asset abstraction. Native SOL is implemented now; SPL Token fields
/// are represented in state so token-vault instructions can be added without a
/// campaign account migration.
pub enum FundingAsset {
    NativeSol,
    SplToken { mint: Pubkey },
}

impl FundingAsset {
    pub fn native_mint() -> Pubkey {
        Pubkey::default()
    }

    pub fn mint(&self) -> Pubkey {
        match self {
            Self::NativeSol => Self::native_mint(),
            Self::SplToken { mint } => *mint,
        }
    }

    pub fn is_native_sol(&self) -> bool {
        matches!(self, Self::NativeSol)
    }
}

pub fn vault_signer_seeds<'a>(campaign: &'a Pubkey, vault_bump: &'a [u8; 1]) -> [&'a [u8]; 3] {
    [VAULT_SEED, campaign.as_ref(), vault_bump]
}
