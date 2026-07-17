use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Signer, transaction::Transaction,
};
use solana_system_interface::program::id as system_program_id;
use vault_raise;

async fn get_campaign(
    banks_client: &mut BanksClient,
    campaign_pda: Pubkey,
) -> vault_raise::Campaign {
    let account = banks_client
        .get_account(campaign_pda)
        .await
        .unwrap()
        .unwrap();
    vault_raise::Campaign::try_deserialize(&mut account.data.as_slice()).unwrap()
}

pub fn program_test() -> ProgramTest {
    let sbf_out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy");
    std::env::set_var("SBF_OUT_DIR", sbf_out_dir);
    ProgramTest::new("vault_raise", vault_raise::id(), None)
}

#[tokio::test]
async fn test_campaign_creation_success() {
    let program_test = program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let campaign_id = 1u64;
    let goal = 10_000_000u64; // 0.01 SOL

    // Get current time from clock
    let clock = banks_client
        .get_sysvar::<solana_sdk::clock::Clock>()
        .await
        .unwrap();
    let deadline = clock.unix_timestamp + 86400; // 1 day in the future

    let (campaign_pda, _) = Pubkey::find_program_address(
        &[
            b"campaign",
            payer.pubkey().as_ref(),
            &campaign_id.to_le_bytes(),
        ],
        &vault_raise::id(),
    );

    let (vault_pda, _) =
        Pubkey::find_program_address(&[b"vault", campaign_pda.as_ref()], &vault_raise::id());

    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::CreateCampaign {
            campaign: campaign_pda,
            vault: vault_pda,
            creator: payer.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::CreateCampaign {
            campaign_id,
            goal,
            deadline,
        }
        .data(),
    };

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Campaign creation should succeed");

    let campaign = get_campaign(&mut banks_client, campaign_pda).await;
    assert_eq!(campaign.creator, payer.pubkey());
    assert_eq!(campaign.goal, goal);
    assert_eq!(campaign.raised, 0);
    assert_eq!(campaign.deadline, deadline);
    assert!(!campaign.claimed);
    assert!(campaign.status == vault_raise::CampaignStatus::Active);
    assert!(campaign.asset == vault_raise::FundingAsset::NativeSol);
    assert_eq!(campaign.metadata_uri_len, 0);
    assert_eq!(campaign.metadata_uri, "");

    let metadata_uri = "ipfs://campaign-metadata".to_string();
    let update_ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::UpdateCampaignMetadata {
            campaign: campaign_pda,
            creator: payer.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::UpdateCampaignMetadata {
            metadata_uri: metadata_uri.clone(),
        }
        .data(),
    };

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut update_tx = Transaction::new_with_payer(&[update_ix], Some(&payer.pubkey()));
    update_tx.sign(&[&payer], recent_blockhash);

    let update_result = banks_client.process_transaction(update_tx).await;
    assert!(update_result.is_ok(), "Metadata update should succeed");

    let campaign = get_campaign(&mut banks_client, campaign_pda).await;
    assert_eq!(campaign.metadata_uri_len, metadata_uri.len() as u16);
    assert_eq!(campaign.metadata_uri, metadata_uri);
}

#[tokio::test]
async fn test_campaign_creation_fails_past_deadline() {
    let program_test = program_test();
    let (banks_client, payer, recent_blockhash) = program_test.start().await;

    let campaign_id = 2u64;
    let goal = 10_000_000u64;
    let deadline = 0; // Past deadline (1970)

    let (campaign_pda, _) = Pubkey::find_program_address(
        &[
            b"campaign",
            payer.pubkey().as_ref(),
            &campaign_id.to_le_bytes(),
        ],
        &vault_raise::id(),
    );

    let (vault_pda, _) =
        Pubkey::find_program_address(&[b"vault", campaign_pda.as_ref()], &vault_raise::id());

    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::CreateCampaign {
            campaign: campaign_pda,
            vault: vault_pda,
            creator: payer.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::CreateCampaign {
            campaign_id,
            goal,
            deadline,
        }
        .data(),
    };

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(
        result.is_err(),
        "Campaign creation should fail due to past deadline"
    );
}

#[tokio::test]
async fn test_campaign_creation_fails_zero_goal() {
    let program_test = program_test();
    let (banks_client, payer, recent_blockhash) = program_test.start().await;

    let campaign_id = 3u64;
    let goal = 0u64; // Zero goal is invalid
    let clock = banks_client
        .get_sysvar::<solana_sdk::clock::Clock>()
        .await
        .unwrap();
    let deadline = clock.unix_timestamp + 86400;

    let (campaign_pda, _) = Pubkey::find_program_address(
        &[
            b"campaign",
            payer.pubkey().as_ref(),
            &campaign_id.to_le_bytes(),
        ],
        &vault_raise::id(),
    );

    let (vault_pda, _) =
        Pubkey::find_program_address(&[b"vault", campaign_pda.as_ref()], &vault_raise::id());

    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::CreateCampaign {
            campaign: campaign_pda,
            vault: vault_pda,
            creator: payer.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::CreateCampaign {
            campaign_id,
            goal,
            deadline,
        }
        .data(),
    };

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(
        result.is_err(),
        "Campaign creation should fail due to zero goal"
    );
}
