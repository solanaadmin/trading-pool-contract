/**
 * pool-info.ts — Read and display current pool state.
 *
 * Usage:
 *   ts-node scripts/pool-info.ts devnet
 *   ts-node scripts/pool-info.ts mainnet
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
  const cluster = process.argv[2] as "devnet" | "mainnet";
  const rpcUrl =
    cluster === "mainnet"
      ? "https://api.mainnet-beta.solana.com"
      : clusterApiUrl("devnet");

  const keypairPath = path.join(os.homedir(), ".config", "solana", "id.json");
  const raw = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
  const deployer = Keypair.fromSecretKey(Uint8Array.from(raw));

  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new anchor.Wallet(deployer);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  const idlPath = path.join(__dirname, "..", "target", "idl", "trading_pool.json");
  const idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));
  const program = new anchor.Program(idl, PROGRAM_ID, provider);

  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), deployer.publicKey.toBuffer()],
    PROGRAM_ID,
  );

  const pool = await (program.account as any).pool.fetch(poolPda);

  const totalDepositedSol = pool.totalDeposited.toNumber() / LAMPORTS_PER_SOL;
  const goalSol = pool.goalLamports.toNumber() / LAMPORTS_PER_SOL;
  const totalProfitSol = pool.totalProfitDeposited.toNumber() / LAMPORTS_PER_SOL;
  const totalPrincipalSol = pool.totalPrincipalReturned.toNumber() / LAMPORTS_PER_SOL;
  const progressPct = ((totalDepositedSol / goalSol) * 100).toFixed(2);

  const [rewardVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("reward_vault"), poolPda.toBuffer()],
    PROGRAM_ID,
  );
  const [principalVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("principal_vault"), poolPda.toBuffer()],
    PROGRAM_ID,
  );

  const rewardVaultBal = await connection.getBalance(rewardVaultPda);
  const principalVaultBal = await connection.getBalance(principalVaultPda);

  console.log("\n════════════════════════════════════════");
  console.log("  TRADING POOL — ON-CHAIN STATUS");
  console.log("════════════════════════════════════════");
  console.log(`Pool address      : ${poolPda.toBase58()}`);
  console.log(`Owner             : ${pool.owner.toBase58()}`);
  console.log(`Trading wallet    : ${pool.tradingWallet.toBase58()}`);
  console.log(`Status            : ${pool.isOpen ? "OPEN — accepting deposits" : "CLOSED"}`);
  console.log("────────────────────────────────────────");
  console.log(`Investors         : ${pool.investorCount}`);
  console.log(`Total deposited   : ${totalDepositedSol.toFixed(2)} SOL`);
  console.log(`Goal              : ${goalSol.toFixed(0)} SOL`);
  console.log(`Progress          : ${progressPct}%`);
  console.log(`Min deposit       : ${pool.minDepositLamports.toNumber() / LAMPORTS_PER_SOL} SOL`);
  console.log("────────────────────────────────────────");
  console.log(`Total profit returned  : ${totalProfitSol.toFixed(4)} SOL`);
  console.log(`Total principal returned: ${totalPrincipalSol.toFixed(4)} SOL`);
  console.log("────────────────────────────────────────");
  console.log(`Reward vault balance   : ${(rewardVaultBal / LAMPORTS_PER_SOL).toFixed(4)} SOL`);
  console.log(`Principal vault balance: ${(principalVaultBal / LAMPORTS_PER_SOL).toFixed(4)} SOL`);
  console.log("════════════════════════════════════════\n");
}

main().catch(console.error);
