/**
 * init.js — Initialize the Trading Pool on mainnet using raw web3.js.
 * No Anchor client dependency — sends the transaction directly.
 * Usage: node scripts/init.js
 */

const {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
} = require("@solana/web3.js");
const crypto = require("crypto");
const fs     = require("fs");
const path   = require("path");
const os     = require("os");

// ─── CONFIG ──────────────────────────────────────────────────────────────────
const PROGRAM_ID     = new PublicKey("H59K28q5kyteWfAGqt34pwkduNwntcambE75tXVp3nZF");
const TRADING_WALLET = new PublicKey("7KCJLv8MdEQV62umoqmjkQ6JNzS4oT8fAd7s3y1EYyQP");
const GOAL_SOL       = BigInt(50_000);
const MIN_DEPOSIT    = BigInt(125);
const RPC_URL        = "https://api.mainnet-beta.solana.com";
// ─────────────────────────────────────────────────────────────────────────────

/** Anchor instruction discriminator = first 8 bytes of sha256("global:<name>") */
function discriminator(name) {
  return crypto.createHash("sha256").update(`global:${name}`).digest().slice(0, 8);
}

/** Write a u64 in little-endian to a Buffer */
function u64LE(n) {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(BigInt(n));
  return buf;
}

async function main() {
  const keypairPath = path.join(os.homedir(), ".config", "solana", "id.json");
  const deployer    = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(keypairPath))));

  console.log("─────────────────────────────────────────────");
  console.log("Cluster        : mainnet-beta");
  console.log("Deployer       :", deployer.publicKey.toBase58());
  console.log("Program ID     :", PROGRAM_ID.toBase58());
  console.log("Trading wallet :", TRADING_WALLET.toBase58());
  console.log("Goal           : 50,000 SOL");
  console.log("Min deposit    : 125 SOL (~$10k)");
  console.log("─────────────────────────────────────────────");

  const connection = new Connection(RPC_URL, "confirmed");
  const balance    = await connection.getBalance(deployer.publicKey);
  console.log("Deployer balance:", balance / LAMPORTS_PER_SOL, "SOL");

  // Derive pool PDA
  const [poolPda, poolBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), deployer.publicKey.toBuffer()],
    PROGRAM_ID,
  );
  console.log("\nPool PDA :", poolPda.toBase58());
  console.log("Pool bump:", poolBump);

  // Check if already initialized
  const existing = await connection.getAccountInfo(poolPda);
  if (existing) {
    console.log("\nPool already initialized. Nothing to do.");
    return;
  }

  // Build instruction data:
  //   [8 bytes discriminator] [32 bytes trading_wallet pubkey] [8 bytes goal_sol u64] [8 bytes min_deposit u64]
  const data = Buffer.concat([
    discriminator("initialize"),
    TRADING_WALLET.toBuffer(),
    u64LE(GOAL_SOL),
    u64LE(MIN_DEPOSIT),
  ]);

  const ix = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deployer.publicKey,          isSigner: true,  isWritable: true  },
      { pubkey: poolPda,                     isSigner: false, isWritable: true  },
      { pubkey: SystemProgram.programId,     isSigner: false, isWritable: false },
    ],
    data,
  });

  const tx = new Transaction().add(ix);

  console.log("\nSending initialize transaction...");
  const sig = await sendAndConfirmTransaction(connection, tx, [deployer], {
    commitment: "confirmed",
  });

  console.log("\nPool initialized successfully!");
  console.log("Tx  :", `https://solscan.io/tx/${sig}`);
  console.log("Pool:", poolPda.toBase58());
  console.log("\nSave the Pool address — needed for all future admin operations.");
}

main().catch((err) => { console.error(err); process.exit(1); });
