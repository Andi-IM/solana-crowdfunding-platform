use anchor_lang::prelude::*;

use crate::errors::VaultRaiseError;

pub const GOVERNANCE_SEED: &[u8] = b"governance";
pub const CAMPAIGN_SEED: &[u8] = b"campaign";
pub const VAULT_SEED: &[u8] = b"vault";
pub const CONTRIBUTION_SEED: &[u8] = b"contribution";
pub const MAX_METADATA_URI_LEN: usize = 200;

const ACCOUNT_DISCRIMINATOR_SIZE: usize = 8;
const PUBKEY_SIZE: usize = 32;
const U64_SIZE: usize = 8;
const I64_SIZE: usize = 8;
const U16_SIZE: usize = 2;
const U8_SIZE: usize = 1;
const BOOL_SIZE: usize = 1;
const STRING_PREFIX_SIZE: usize = 4;

#[account]
#[derive(InitSpace)]
/// Persistent campaign state.
pub struct Campaign {
    pub creator: Pubkey,
    pub goal: u64,
    pub raised: u64,
    pub refunded: u64,
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
    pub const SPACE: usize = ACCOUNT_DISCRIMINATOR_SIZE + Self::INIT_SPACE;

    pub fn realloc_space(metadata_uri_len: usize) -> usize {
        ACCOUNT_DISCRIMINATOR_SIZE
            + PUBKEY_SIZE
            + U64_SIZE
            + U64_SIZE
            + U64_SIZE
            + I64_SIZE
            + BOOL_SIZE
            + U8_SIZE
            + FundingAsset::INIT_SPACE
            + U16_SIZE
            + U8_SIZE
            + U8_SIZE
            + STRING_PREFIX_SIZE
            + metadata_uri_len
    }

    pub fn ensure_contributable(&self, current_time: i64) -> Result<()> {
        self.ensure_native_sol()?;
        require!(current_time < self.deadline, VaultRaiseError::CampaignEnded);
        require!(!self.claimed, VaultRaiseError::AlreadyClaimed);
        self.ensure_active()
    }

    pub fn ensure_withdrawable(&self, current_time: i64) -> Result<()> {
        self.ensure_native_sol()?;
        require!(
            self.raised >= self.goal,
            VaultRaiseError::CampaignNotSuccessful
        );
        require!(
            current_time >= self.deadline,
            VaultRaiseError::CampaignNotEnded
        );
        require!(!self.claimed, VaultRaiseError::AlreadyClaimed);
        Ok(())
    }

    pub fn ensure_refundable(&self, current_time: i64) -> Result<()> {
        self.ensure_native_sol()?;
        require!(self.raised < self.goal, VaultRaiseError::CampaignNotFailed);
        require!(
            current_time >= self.deadline,
            VaultRaiseError::CampaignNotEnded
        );
        self.ensure_active()
    }

    pub fn mark_claimed(&mut self) {
        self.claimed = true;
        self.status = CampaignStatus::Claimed;
    }

    pub fn tracked_outstanding(&self) -> Result<u64> {
        self.raised
            .checked_sub(self.refunded)
            .ok_or(VaultRaiseError::ArithmeticOverflow.into())
    }

    fn ensure_native_sol(&self) -> Result<()> {
        require!(
            self.asset.is_native_sol(),
            VaultRaiseError::NativeAssetRequired
        );
        Ok(())
    }

    fn ensure_active(&self) -> Result<()> {
        require!(
            self.status == CampaignStatus::Active,
            VaultRaiseError::CampaignNotActive
        );
        Ok(())
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
    pub const SPACE: usize = ACCOUNT_DISCRIMINATOR_SIZE + Self::INIT_SPACE;
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
    pub const SPACE: usize = ACCOUNT_DISCRIMINATOR_SIZE + Self::INIT_SPACE;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
/// Explicit lifecycle marker for campaign state transitions.
pub enum CampaignStatus {
    Active,
    Claimed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
/// Funding asset state. Native SOL is operational now; SPL Token is reserved so
/// future token-vault instructions can be added without a campaign migration.
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
