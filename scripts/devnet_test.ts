import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import fs from "fs";
import path from "path";

// Load IDL
const idl = JSON.parse(fs.readFileSync(path.resolve(__dirname, "../target/idl/vault_raise.json"), "utf8"));
const PROGRAM_ID = new PublicKey("GeYMy79EJmUs8japokaVcadb2RRs6vv7c4xYE2fbjkQW");

async function main() {
    console.log("Starting Devnet Test...");

    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    const secretKey = JSON.parse(fs.readFileSync(path.resolve(__dirname, "../id.json"), "utf8"));
    const payer = Keypair.fromSecretKey(new Uint8Array(secretKey));

    const wallet = new anchor.Wallet(payer);
    const provider = new anchor.AnchorProvider(connection, wallet, {
        preflightCommitment: "confirmed",
    });
    anchor.setProvider(provider);

    const program = new Program(idl, provider);

    // ==========================================
    // Scenario 1: Successful Campaign
    // ==========================================
    console.log("\n--- Scenario 1: Successful Campaign ---");
    const successCampaignId = new anchor.BN(Math.floor(Math.random() * 1000000));
    const goal = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL); 
    const futureDeadline = new anchor.BN(Math.floor(Date.now() / 1000) + 15); // Deadline is 15s in the future

    const [successCampaignPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("campaign"), payer.publicKey.toBuffer(), successCampaignId.toArrayLike(Buffer, "le", 8)],
        program.programId
    );
    const [successVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), successCampaignPda.toBuffer()],
        program.programId
    );

    const txCreate1 = await program.methods
        .createCampaign(successCampaignId, goal, futureDeadline)
        .accounts({
            campaign: successCampaignPda,
            vault: successVaultPda,
            creator: payer.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    console.log("Create Campaign (Success) TX:", txCreate1);

    const [successContributionPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("contribution"), successCampaignPda.toBuffer(), payer.publicKey.toBuffer()],
        program.programId
    );

    const txContribute1 = await program.methods
        .contribute(goal) // meet the goal exactly
        .accounts({
            campaign: successCampaignPda,
            contribution: successContributionPda,
            vault: successVaultPda,
            donor: payer.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    console.log("Contribute (Success) TX:", txContribute1);

    console.log("Waiting 20 seconds for deadline to pass...");
    await new Promise(r => setTimeout(r, 20000));

    const txWithdraw = await program.methods
        .withdraw()
        .accounts({
            campaign: successCampaignPda,
            vault: successVaultPda,
            creator: payer.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    console.log("Withdraw (Success) TX:", txWithdraw);

    // ==========================================
    // Scenario 2: Failed Campaign (Refund)
    // ==========================================
    console.log("\n--- Scenario 2: Failed Campaign (Refund) ---");
    const failedCampaignId = new anchor.BN(Math.floor(Math.random() * 1000000));
    const highGoal = new anchor.BN(10 * anchor.web3.LAMPORTS_PER_SOL); 
    const futureDeadline2 = new anchor.BN(Math.floor(Date.now() / 1000) + 15);

    const [failedCampaignPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("campaign"), payer.publicKey.toBuffer(), failedCampaignId.toArrayLike(Buffer, "le", 8)],
        program.programId
    );
    const [failedVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), failedCampaignPda.toBuffer()],
        program.programId
    );

    const txCreate2 = await program.methods
        .createCampaign(failedCampaignId, highGoal, futureDeadline2)
        .accounts({
            campaign: failedCampaignPda,
            vault: failedVaultPda,
            creator: payer.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    console.log("Create Campaign (Failed) TX:", txCreate2);

    const [failedContributionPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("contribution"), failedCampaignPda.toBuffer(), payer.publicKey.toBuffer()],
        program.programId
    );

    const txContribute2 = await program.methods
        .contribute(new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL)) // didn't meet goal
        .accounts({
            campaign: failedCampaignPda,
            contribution: failedContributionPda,
            vault: failedVaultPda,
            donor: payer.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    console.log("Contribute (Failed) TX:", txContribute2);

    console.log("Waiting 20 seconds for deadline to pass...");
    await new Promise(r => setTimeout(r, 20000));

    const txRefund = await program.methods
        .refund()
        .accounts({
            campaign: failedCampaignPda,
            contribution: failedContributionPda,
            vault: failedVaultPda,
            donor: payer.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    console.log("Refund (Failed) TX:", txRefund);

    console.log("\nDone!");
}

main().catch(err => {
    console.error(err);
});
