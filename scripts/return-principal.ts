/**
 * return-principal.ts — Return an investor's principal into the vault.
 *
 * Usage:
 *   ts-node scripts/return-principal.ts <investor_wallet> <devnet|mainnet>
 *
 * Example:
 *   ts-node scripts/return-principal.ts 7KCJLv8M...EYyQP mainnet
 *
 * Call this for each investor whose lock period is expiring.
 * After this, the investor can call withdraw_principal from the frontend.
 */

import * as anchor from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  clusterApiUrl,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

const PROGRAM_ID = new PublicKey("REPLACE_WITH_YOUR_PROGRAM_ID");

async function main() {
  const investorAddress = process.argv[2];
  const cluster = process.argv[3] as "devnet" | "mainnet";

  if (!investorAddress || !["devnet", "mainnet"].includes(cluster)) {
    console.error("Usage: ts-node scripts/return-principal.ts <investor_wallet> <devnet|mainnet>");
    process.exit(1);
  }

  const investorPubkey = new PublicKey(investorAddress);
  const rpcUrl =
    cluster === "mainnet"
      ? "https://api.mainnet-beta.solana.com"
      : clusterApiUrl("devnet");

  const keypairPath = path.join(os.homedir(), ".config", "solana", "id.json");
  const raw = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
  const owner = Keypair.fromSecretKey(Uint8Array.from(raw));

  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new anchor.Wallet(owner);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  const idlPath = path.join(__dirname, "..", "target", "idl", "trading_pool.json");
  const idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));
  const program = new anchor.Program(idl, PROGRAM_ID, provider);

  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), owner.publicKey.toBuffer()],
    PROGRAM_ID,
  );
  const [principalVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("principal_vault"), poolPda.toBuffer()],
    PROGRAM_ID,
  );
  const [stakePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("stake"), poolPda.toBuffer(), investorPubkey.toBuffer()],
    PROGRAM_ID,
  );

  // Fetch stake info
  const stake = await (program.account as any).stake.fetch(stakePda);
  const amountSol = stake.amountLamports.toNumber() / LAMPORTS_PER_SOL;
  const lockEnd = new Date(stake.lockEndTime.toNumber() * 1000);
  const now = new Date();
  const isUnlocked = now >= lockEnd;

  console.log("─────────────────────────────────────────────");
  console.log(`Investor          : ${investorPubkey.toBase58()}`);
  console.log(`Principal amount  : ${amountSol.toFixed(4)} SOL`);
  console.log(`Lock end time     : ${lockEnd.toISOString()}`);
  console.log(`Status            : ${isUnlocked ? "UNLOCKED - ready" : "Still locked (returning early)"}`);
  console.log(`Receiving wallet  : ${stake.receivingWallet.toBase58()}`);
  console.log("─────────────────────────────────────────────");

  if (cluster === "mainnet") {
    console.log(`\nAbout to send ${amountSol} SOL from your wallet into the principal vault.`);
    console.log("Press Enter to confirm, Ctrl+C to abort.");
    await waitForEnter();
  }

  const tx = await program.methods
    .returnPrincipal()
    .accounts({
      owner:          owner.publicKey,
      pool:           poolPda,
      principalVault: principalVaultPda,
      stake:          stakePda,
    })
    .signers([owner])
    .rpc();

  console.log(`\nPrincipal returned: ${amountSol} SOL deposited into vault`);
  console.log(`Transaction: https://solscan.io/tx/${tx}${cluster === "devnet" ? "?cluster=devnet" : ""}`);
  console.log(`Investor can now call withdraw_principal from the frontend.`);
}

function waitForEnter(): Promise<void> {
  return new Promise((resolve) => {
    process.stdin.setRawMode(true);
    process.stdin.resume();
    process.stdin.once("data", () => {
      process.stdin.setRawMode(false);
      process.stdin.pause();
      resolve();
    });
  });
}

main().catch(console.error);
