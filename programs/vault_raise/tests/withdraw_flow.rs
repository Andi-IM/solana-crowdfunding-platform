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

pub fn program_test() -> ProgramTest {
    let sbf_out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy");
    std::env::set_var("SBF_OUT_DIR", sbf_out_dir);
    ProgramTest::new("vault_raise", vault_raise::id(), None)
}

async fn setup_funded_campaign(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    campaign_id: u64,
    goal: u64,
    deadline_offset: i64,
    amount_to_fund: u64,
) -> (Pubkey, Pubkey, Keypair, Pubkey) {
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

    // Fund the campaign
    let donor = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &payer.pubkey(),
        &donor.pubkey(),
        amount_to_fund + 1_000_000_000,
    );
    let mut fund_tx = Transaction::new_with_payer(&[fund_ix], Some(&payer.pubkey()));
    fund_tx.sign(&[payer], context.last_blockhash);
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

    let contribute_ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::Contribute {
            campaign: campaign_pda,
            contribution: contribution_pda,
            vault: vault_pda,
            donor: donor.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::Contribute {
            amount: amount_to_fund,
        }
        .data(),
    };

    let mut contribute_tx = Transaction::new_with_payer(&[contribute_ix], Some(&donor.pubkey()));
    contribute_tx.sign(&[&donor], context.last_blockhash);
    context
        .banks_client
        .process_transaction(contribute_tx)
        .await
        .unwrap();

    (campaign_pda, vault_pda, donor, contribution_pda)
}

#[tokio::test]
async fn test_withdraw_success_and_twice_fails() {
    let mut context = program_test().start_with_context().await;
    let payer = Keypair::try_from(context.payer.to_bytes().as_ref()).unwrap();

    let campaign_id = 1u64;
    let goal = 1000 * 1_000_000_000;

    // Setup campaign that meets the goal
    let (campaign_pda, vault_pda, _, _) = setup_funded_campaign(
        &mut context,
        &payer,
        campaign_id,
        goal,
        100, // +100s deadline
        1500 * 1_000_000_000,
    )
    .await;

    // Fast-forward time to past deadline
    let mut clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    clock.unix_timestamp += 200;
    context.set_sysvar(&clock);

    let balance_before = context
        .banks_client
        .get_balance(payer.pubkey())
        .await
        .unwrap();

    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::Withdraw {
            campaign: campaign_pda,
            vault: vault_pda,
            creator: payer.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::Withdraw {}.data(),
    };

    let mut tx = Transaction::new_with_payer(&[ix.clone()], Some(&payer.pubkey()));
    tx.sign(&[&payer], context.last_blockhash);
    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Withdraw should succeed");

    let balance_after = context
        .banks_client
        .get_balance(payer.pubkey())
        .await
        .unwrap();

    // Balance should increase (excluding transaction fees) by exactly 1500 SOL
    let expected_increase = 1500 * 1_000_000_000;
    assert!(balance_after > balance_before);
    assert!(balance_after - balance_before > expected_increase - 10_000); // 10k margin for tx fees

    let campaign = get_campaign(&mut context.banks_client, campaign_pda).await;
    assert_eq!(campaign.goal, goal);
    assert_eq!(campaign.raised, expected_increase);
    assert!(campaign.claimed);
    assert!(campaign.status == vault_raise::CampaignStatus::Claimed);

    // Second withdraw should fail (AlreadyClaimed)
    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let mut tx2 = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx2.sign(&[&payer], recent_blockhash);
    let result2 = context.banks_client.process_transaction(tx2).await;
    assert!(result2.is_err(), "Second withdraw should fail");
}

#[tokio::test]
async fn test_withdraw_fails_before_deadline() {
    let mut context = program_test().start_with_context().await;
    let payer = Keypair::try_from(context.payer.to_bytes().as_ref()).unwrap();

    let campaign_id = 2u64;
    let goal = 1000 * 1_000_000_000;

    // Setup campaign that meets the goal but hasn't reached deadline
    let (campaign_pda, vault_pda, _, _) = setup_funded_campaign(
        &mut context,
        &payer,
        campaign_id,
        goal,
        86400, // 1 day in the future
        1500 * 1_000_000_000,
    )
    .await;

    // Try withdraw (before deadline)
    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::Withdraw {
            campaign: campaign_pda,
            vault: vault_pda,
            creator: payer.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::Withdraw {}.data(),
    };

    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], context.last_blockhash);
    let result = context.banks_client.process_transaction(tx).await;
    assert!(result.is_err(), "Withdraw before deadline should fail");
}

#[tokio::test]
async fn test_withdraw_by_non_creator_fails() {
    let mut context = program_test().start_with_context().await;
    let payer = Keypair::try_from(context.payer.to_bytes().as_ref()).unwrap();

    let campaign_id = 3u64;
    let goal = 1000 * 1_000_000_000;

    let (campaign_pda, vault_pda, _, _) = setup_funded_campaign(
        &mut context,
        &payer,
        campaign_id,
        goal,
        100,
        1500 * 1_000_000_000,
    )
    .await;

    // Fast-forward time to past deadline
    let mut clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    clock.unix_timestamp += 200;
    context.set_sysvar(&clock);

    // Some random hacker account
    let hacker = Keypair::new();
    let fund_ix =
        solana_sdk::system_instruction::transfer(&payer.pubkey(), &hacker.pubkey(), 1_000_000_000);
    let mut fund_tx = Transaction::new_with_payer(&[fund_ix], Some(&payer.pubkey()));
    fund_tx.sign(&[&payer], context.last_blockhash);
    context
        .banks_client
        .process_transaction(fund_tx)
        .await
        .unwrap();

    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::Withdraw {
            campaign: campaign_pda,
            vault: vault_pda,
            creator: hacker.pubkey(), // Mismatched creator
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::Withdraw {}.data(),
    };

    let mut tx = Transaction::new_with_payer(&[ix], Some(&hacker.pubkey()));
    tx.sign(&[&hacker], context.last_blockhash);
    let result = context.banks_client.process_transaction(tx).await;
    assert!(result.is_err(), "Withdraw by a non-creator should fail");
}
