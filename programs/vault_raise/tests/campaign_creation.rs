use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Signer, signer::keypair::Keypair,
    system_program, transaction::Transaction,
};
use vault_raise;

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [solana_sdk::account_info::AccountInfo<'a>],
    instruction_data: &[u8],
) -> solana_sdk::entrypoint::ProgramResult {
    vault_raise::entry(program_id, accounts, instruction_data)
}

pub fn program_test() -> ProgramTest {
    ProgramTest::new("vault_raise", vault_raise::id(), processor!(process_instruction))
}

#[tokio::test]
async fn test_campaign_creation_success() {
    let mut program_test = program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let campaign_id = 1u64;
    let goal = 10_000_000u64; // 0.01 SOL
    
    // Get current time from clock
    let clock = banks_client.get_sysvar::<solana_sdk::clock::Clock>().await.unwrap();
    let deadline = clock.unix_timestamp + 86400; // 1 day in the future

    let (campaign_pda, _) = Pubkey::find_program_address(
        &[
            b"campaign",
            payer.pubkey().as_ref(),
            &campaign_id.to_le_bytes(),
        ],
        &vault_raise::id(),
    );

    let (vault_pda, _) = Pubkey::find_program_address(
        &[b"vault", campaign_pda.as_ref()],
        &vault_raise::id(),
    );

    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::CreateCampaign {
            campaign: campaign_pda,
            vault: vault_pda,
            creator: payer.pubkey(),
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::CreateCampaign {
            _campaign_id: campaign_id,
            goal,
            deadline,
        }
        .data(),
    };

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Campaign creation should succeed");

    // Verify state
    let account = banks_client.get_account(campaign_pda).await.unwrap().unwrap();
    // In a real test, we would deserialize the Anchor account here, but verifying it exists is a good first step.
    assert!(account.data.len() > 0);
}

#[tokio::test]
async fn test_campaign_creation_fails_past_deadline() {
    let mut program_test = program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

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

    let (vault_pda, _) = Pubkey::find_program_address(
        &[b"vault", campaign_pda.as_ref()],
        &vault_raise::id(),
    );

    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::CreateCampaign {
            campaign: campaign_pda,
            vault: vault_pda,
            creator: payer.pubkey(),
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::CreateCampaign {
            _campaign_id: campaign_id,
            goal,
            deadline,
        }
        .data(),
    };

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err(), "Campaign creation should fail due to past deadline");
}

#[tokio::test]
async fn test_campaign_creation_fails_zero_goal() {
    let mut program_test = program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let campaign_id = 3u64;
    let goal = 0u64; // Zero goal is invalid
    let clock = banks_client.get_sysvar::<solana_sdk::clock::Clock>().await.unwrap();
    let deadline = clock.unix_timestamp + 86400;

    let (campaign_pda, _) = Pubkey::find_program_address(
        &[
            b"campaign",
            payer.pubkey().as_ref(),
            &campaign_id.to_le_bytes(),
        ],
        &vault_raise::id(),
    );

    let (vault_pda, _) = Pubkey::find_program_address(
        &[b"vault", campaign_pda.as_ref()],
        &vault_raise::id(),
    );

    let ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::CreateCampaign {
            campaign: campaign_pda,
            vault: vault_pda,
            creator: payer.pubkey(),
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::CreateCampaign {
            _campaign_id: campaign_id,
            goal,
            deadline,
        }
        .data(),
    };

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err(), "Campaign creation should fail due to zero goal");
}
