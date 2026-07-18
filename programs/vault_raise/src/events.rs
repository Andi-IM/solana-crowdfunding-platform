use anchor_lang::prelude::*;

#[event]
pub struct CampaignCreated {
    pub campaign: Pubkey,
    pub creator: Pubkey,
    pub campaign_id: u64,
    pub goal: u64,
    pub deadline: i64,
    pub vault: Pubkey,
    pub asset: Pubkey,
}

#[event]
pub struct CampaignMetadataUpdated {
    pub campaign: Pubkey,
    pub creator: Pubkey,
    pub metadata_uri: String,
}

#[event]
pub struct CampaignContributed {
    pub campaign: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
    pub total_raised: u64,
}

#[event]
pub struct CampaignWithdrawn {
    pub campaign: Pubkey,
    pub creator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct VaultSurplusSwept {
    pub campaign: Pubkey,
    pub creator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ContributionRefunded {
    pub campaign: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CampaignClosed {
    pub campaign: Pubkey,
    pub creator: Pubkey,
}

#[event]
pub struct ContributionClosed {
    pub campaign: Pubkey,
    pub donor: Pubkey,
}

#[event]
pub struct GovernanceInitialized {
    pub governance: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct GovernanceTransferred {
    pub governance: Pubkey,
    pub previous_authority: Pubkey,
    pub new_authority: Pubkey,
}
