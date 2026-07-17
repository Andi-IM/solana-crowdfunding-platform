use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Signer, signer::keypair::Keypair,
    transaction::Transaction,
};
use solana_system_interface::program::id as system_program_id;
use vault_raise;

async fn get_governance(
    banks_client: &mut BanksClient,
    governance_pda: Pubkey,
) -> vault_raise::Governance {
    let account = banks_client
        .get_account(governance_pda)
        .await
        .unwrap()
        .unwrap();
    vault_raise::Governance::try_deserialize(&mut account.data.as_slice()).unwrap()
}

pub fn program_test() -> ProgramTest {
    let sbf_out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy");
    std::env::set_var("SBF_OUT_DIR", sbf_out_dir);
    ProgramTest::new("vault_raise", vault_raise::id(), None)
}

#[tokio::test]
async fn test_initialize_and_transfer_governance() {
    let mut context = program_test().start_with_context().await;
    let authority = Keypair::try_from(context.payer.to_bytes().as_ref()).unwrap();
    let new_authority = Keypair::new();

    let (governance_pda, _) = Pubkey::find_program_address(&[b"governance"], &vault_raise::id());

    let init_ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::InitializeGovernance {
            governance: governance_pda,
            authority: authority.pubkey(),
            system_program: system_program_id(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::InitializeGovernance {}.data(),
    };

    let mut init_tx = Transaction::new_with_payer(&[init_ix], Some(&authority.pubkey()));
    init_tx.sign(&[&authority], context.last_blockhash);
    context
        .banks_client
        .process_transaction(init_tx)
        .await
        .expect("Governance initialization should succeed");

    let governance = get_governance(&mut context.banks_client, governance_pda).await;
    assert_eq!(governance.authority, authority.pubkey());
    assert_eq!(governance.pending_authority, Pubkey::default());

    let transfer_ix = Instruction {
        program_id: vault_raise::id(),
        accounts: vault_raise::accounts::TransferGovernance {
            governance: governance_pda,
            authority: authority.pubkey(),
        }
        .to_account_metas(None),
        data: vault_raise::instruction::TransferGovernance {
            new_authority: new_authority.pubkey(),
        }
        .data(),
    };

    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let mut transfer_tx = Transaction::new_with_payer(&[transfer_ix], Some(&authority.pubkey()));
    transfer_tx.sign(&[&authority], recent_blockhash);
    context
        .banks_client
        .process_transaction(transfer_tx)
        .await
        .expect("Governance transfer should succeed");

    let governance = get_governance(&mut context.banks_client, governance_pda).await;
    assert_eq!(governance.authority, new_authority.pubkey());
    assert_eq!(governance.pending_authority, Pubkey::default());
}
