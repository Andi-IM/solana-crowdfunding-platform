use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program_test::*;
use solana_sdk::{
    clock::Clock, instruction::Instruction, pubkey::Pubkey, signature::Signer,
    signer::keypair::Keypair, transaction::Transaction,
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

async fn get_contribution(
    banks_client: &mut BanksClient,
    contribution_pda: Pubkey,
) -> vault_raise::Contribution {
    let account = banks_client
        .get_account(contribution_pda)
        .await
        .unwrap()
        .unwrap();
    vault_raise::Contribution::try_deserialize(&mut account.data.as_slice()).unwrap()
}

pub fn program_test() -> ProgramTest {
    let sbf_out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy");
    std::env::set_var("SBF_OUT_DIR", sbf_out_dir);
    ProgramTest::new("vault_raise", vault_raise::id(), None)
}

async fn setup_campaign(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    campaign_id: u64,
    goal: u64,
    deadline_offset: i64,
) -> (Pubkey, Pubkey) {
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let deadline = clock.unix_timestamp + deadline_offset;

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
    transaction.sign(&[payer], context.last_blockhash);
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    (campaign_pda, vault_pda)
}

#[tokio::test]
async fn test_contribution_flow_success() {
    let mut context = program_test().start_with_context().await;
    // Clone payer keypair
    let payer = Keypair::try_from(context.payer.to_bytes().as_ref()).unwrap();

    let campaign_id = 1u64;
    let goal = 1000 * 1_000_000_000;
    let (campaign_pda, vault_pda) =
        setup_campaign(&mut context, &payer, campaign_id, goal, 86400).await;

    let donor = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &payer.pubkey(),
        &donor.pubkey(),
        2000 * 1_000_000_000,
    );
    let mut fund_tx = Transaction::new_with_payer(&[fund_ix], Some(&payer.pubkey()));
    fund_tx.sign(&[&payer], context.last_blockhash);
    context
        .banks_client
        .process_transaction(fund_tx)
        .await
        .unwrap();

    let (contribution_pda, _) = Pubkey::find_program_address(
        &[
            b"contribution",
            campaign_pda.as_ref(),
            donor.pubkey().as_ref(),
        ],
        &vault_raise::id(),
    );

    let amount_1 = 600 * 1_000_000_000;
    let ix1 = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::Contribute {
            campaign: campaign_pda,
            contribution: contribution_pda,
            vault: vault_pda,
            donor: donor.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::Contribute { amount: amount_1 }.data(),
    };

    let mut tx1 = Transaction::new_with_payer(&[ix1], Some(&donor.pubkey()));
    tx1.sign(&[&donor], context.last_blockhash);
    context
        .banks_client
        .process_transaction(tx1)
        .await
        .expect("First contribution should succeed");

    let amount_2 = 500 * 1_000_000_000;
    let ix2 = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::Contribute {
            campaign: campaign_pda,
            contribution: contribution_pda,
            vault: vault_pda,
            donor: donor.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::Contribute { amount: amount_2 }.data(),
    };

    let mut tx2 = Transaction::new_with_payer(&[ix2], Some(&donor.pubkey()));
    tx2.sign(&[&donor], context.last_blockhash);
    context
        .banks_client
        .process_transaction(tx2)
        .await
        .expect("Second contribution should succeed");

    let campaign = get_campaign(&mut context.banks_client, campaign_pda).await;
    assert_eq!(campaign.goal, goal);
    assert_eq!(campaign.raised, amount_1 + amount_2);
    assert!(!campaign.claimed);
    assert!(campaign.status == vault_raise::CampaignStatus::Active);

    let contribution = get_contribution(&mut context.banks_client, contribution_pda).await;
    assert_eq!(contribution.campaign, campaign_pda);
    assert_eq!(contribution.donor, donor.pubkey());
    assert_eq!(contribution.amount, amount_1 + amount_2);
    assert!(!contribution.refunded);
}

#[tokio::test]
async fn test_contribution_fails_past_deadline() {
    let mut context = program_test().start_with_context().await;
    let payer = Keypair::try_from(context.payer.to_bytes().as_ref()).unwrap();

    let campaign_id = 2u64;
    let goal = 1000 * 1_000_000_000;
    let (campaign_pda, vault_pda) =
        setup_campaign(&mut context, &payer, campaign_id, goal, 100).await;

    let donor = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &payer.pubkey(),
        &donor.pubkey(),
        2000 * 1_000_000_000,
    );
    let mut fund_tx = Transaction::new_with_payer(&[fund_ix], Some(&payer.pubkey()));
    fund_tx.sign(&[&payer], context.last_blockhash);
    context
        .banks_client
        .process_transaction(fund_tx)
        .await
        .unwrap();

    // Warp the clock forward
    let mut clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    clock.unix_timestamp += 200;
    context.set_sysvar(&clock);

    let (contribution_pda, _) = Pubkey::find_program_address(
        &[
            b"contribution",
            campaign_pda.as_ref(),
            donor.pubkey().as_ref(),
        ],
        &vault_raise::id(),
    );

    let amount = 100 * 1_000_000_000;
    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::Contribute {
            campaign: campaign_pda,
            contribution: contribution_pda,
            vault: vault_pda,
            donor: donor.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::Contribute { amount }.data(),
    };

    let mut tx = Transaction::new_with_payer(&[ix], Some(&donor.pubkey()));
    tx.sign(&[&donor], context.last_blockhash);

    let result = context.banks_client.process_transaction(tx).await;
    assert!(result.is_err(), "Contribution should fail after deadline");
}

#[tokio::test]
async fn test_contribution_fails_zero_amount() {
    let mut context = program_test().start_with_context().await;
    let payer = Keypair::try_from(context.payer.to_bytes().as_ref()).unwrap();

    let campaign_id = 3u64;
    let goal = 1000 * 1_000_000_000;
    let (campaign_pda, vault_pda) =
        setup_campaign(&mut context, &payer, campaign_id, goal, 86400).await;

    let donor = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &payer.pubkey(),
        &donor.pubkey(),
        2000 * 1_000_000_000,
    );
    let mut fund_tx = Transaction::new_with_payer(&[fund_ix], Some(&payer.pubkey()));
    fund_tx.sign(&[&payer], context.last_blockhash);
    context
        .banks_client
        .process_transaction(fund_tx)
        .await
        .unwrap();

    let (contribution_pda, _) = Pubkey::find_program_address(
        &[
            b"contribution",
            campaign_pda.as_ref(),
            donor.pubkey().as_ref(),
        ],
        &vault_raise::id(),
    );

    let amount = 0; // Zero amount
    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::Contribute {
            campaign: campaign_pda,
            contribution: contribution_pda,
            vault: vault_pda,
            donor: donor.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::Contribute { amount }.data(),
    };

    let mut tx = Transaction::new_with_payer(&[ix], Some(&donor.pubkey()));
    tx.sign(&[&donor], context.last_blockhash);

    let result = context.banks_client.process_transaction(tx).await;
    assert!(result.is_err(), "Contribution should fail if amount is 0");
}
