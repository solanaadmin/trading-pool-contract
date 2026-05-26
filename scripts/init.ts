/**
 * init.ts — Initialize the Trading Pool on mainnet.
 * Usage: ts-node scripts/init.ts mainnet
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

// ─────────────────────────────────────────────────────────────────────────────
//  CONFIGURATION
// ─────────────────────────────────────────────────────────────────────────────

const PROGRAM_ID = new PublicKey("H59K28q5kyteWfAGqt34pwkduNwntcambE75tXVp3nZF");

const CONFIG = {
  TRADING_WALLET:  new PublicKey("7KCJLv8MdEQV62umoqmjkQ6JNzS4oT8fAd7s3y1EYyQP"),
  GOAL_SOL:        new anchor.BN(50_000),
  MIN_DEPOSIT_SOL: new anchor.BN(125),
};

// ─────────────────────────────────────────────────────────────────────────────

async function main() {
  const cluster = (process.argv[2] as string) || "mainnet";
  const rpcUrl =
    cluster === "mainnet"
      ? "https://api.mainnet-beta.solana.com"
      : clusterApiUrl("devnet");

  const keypairPath = path.join(os.homedir(), ".config", "solana", "id.json");
  const raw = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
  const deployer = Keypair.fromSecretKey(Uint8Array.from(raw));

  console.log("─────────────────────────────────────────────");
  console.log(`Cluster        : ${cluster}`);
  console.log(`Deployer       : ${deployer.publicKey.toBase58()}`);
  console.log(`Program ID     : ${PROGRAM_ID.toBase58()}`);
  console.log(`Trading wallet : ${CONFIG.TRADING_WALLET.toBase58()}`);
  console.log(`Goal           : ${CONFIG.GOAL_SOL.toString()} SOL`);
  console.log(`Min deposit    : ${CONFIG.MIN_DEPOSIT_SOL.toString()} SOL`);
  console.log("─────────────────────────────────────────────");

  const connection = new Connection(rpcUrl, "confirmed");
  const balance = await connection.getBalance(deployer.publicKey);
  console.log(`Deployer balance: ${balance / LAMPORTS_PER_SOL} SOL`);

  const wallet   = new anchor.Wallet(deployer);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  const idlPath = path.join(__dirname, "..", "target", "idl", "trading_pool.json");
  const idl     = JSON.parse(fs.readFileSync(idlPath, "utf-8"));

  // @coral-xyz/anchor >=0.29: pass idl + programId separately
  const program = new anchor.Program(idl, PROGRAM_ID, provider) as anchor.Program;

  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), deployer.publicKey.toBuffer()],
    PROGRAM_ID,
  );

  console.log(`\nPool PDA : ${poolPda.toBase58()}`);

  const existing = await connection.getAccountInfo(poolPda);
  if (existing !== null) {
    console.log("Pool already initialized. Nothing to do.");
    process.exit(0);
  }

  console.log("\nSending initialize transaction...");
  const tx = await (program.methods as any)
    .initialize(CONFIG.TRADING_WALLET, CONFIG.GOAL_SOL, CONFIG.MIN_DEPOSIT_SOL)
    .accounts({
      owner:         deployer.publicKey,
      pool:          poolPda,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([deployer])
    .rpc();

  console.log(`\nPool initialized!`);
  console.log(`Tx     : https://solscan.io/tx/${tx}`);
  console.log(`Pool   : ${poolPda.toBase58()}`);
  console.log("\nSave the Pool address — needed for all admin operations.");
}

main().catch((err) => { console.error(err); process.exit(1); });
