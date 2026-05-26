/**
 * deposit-profit.ts — Owner deposits trading profit into the reward vault.
 *
 * Usage:
 *   ts-node scripts/deposit-profit.ts <amount_sol> <devnet|mainnet>
 *
 * Example — deposit 500 SOL profit on mainnet:
 *   ts-node scripts/deposit-profit.ts 500 mainnet
 *
 * This single transaction updates every investor's claimable balance.
 * Investors then pull their share at any time via claim_rewards.
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
  const amountSol = parseFloat(process.argv[2]);
  const cluster = process.argv[3] as "devnet" | "mainnet";

  if (isNaN(amountSol) || amountSol <= 0 || !["devnet", "mainnet"].includes(cluster)) {
    console.error("Usage: ts-node scripts/deposit-profit.ts <amount_sol> <devnet|mainnet>");
    process.exit(1);
  }

  const amountLamports = new anchor.BN(Math.floor(amountSol * LAMPORTS_PER_SOL));

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
  const [rewardVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("reward_vault"), poolPda.toBuffer()],
    PROGRAM_ID,
  );

  const pool = await (program.account as any).pool.fetch(poolPda);
  const totalDepositedSol = pool.totalDeposited.toNumber() / LAMPORTS_PER_SOL;

  console.log("─────────────────────────────────────────────");
  console.log(`Depositing profit : ${amountSol} SOL`);
  console.log(`Total staked      : ${totalDepositedSol.toFixed(2)} SOL`);
  console.log(`Investors         : ${pool.investorCount}`);
  console.log(`Each investor gets: proportional share`);
  console.log("─────────────────────────────────────────────");

  if (cluster === "mainnet") {
    console.log("\nMAINNET — press Enter to confirm, Ctrl+C to abort.");
    await waitForEnter();
  }

  const tx = await program.methods
    .depositProfit(amountLamports)
    .accounts({
      owner:       owner.publicKey,
      pool:        poolPda,
      rewardVault: rewardVaultPda,
    })
    .signers([owner])
    .rpc();

  console.log(`\nProfit deposited: ${amountSol} SOL`);
  console.log(`Transaction: https://solscan.io/tx/${tx}${cluster === "devnet" ? "?cluster=devnet" : ""}`);
  console.log(`All ${pool.investorCount} investors can now claim their share.`);
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
