# Trading Pool — Solana Smart Contract

A transparent SOL trading fund. Investors deposit SOL directly into the owner's
trading wallet. The smart contract records every obligation on-chain and handles
profit distribution and principal return via a gas-efficient pull model.

---

## Architecture

```
Investor  ──deposit──►  Owner's trading wallet   (SOL moves here immediately)
                  │
                  └──►  Stake PDA                 (on-chain obligation recorded)

Owner     ──deposit_profit──►  Reward Vault PDA   (profit held for claims)
Owner     ──return_principal──► Principal Vault PDA (capital held for withdrawal)

Investor  ──claim_rewards──►   Reward Vault PDA → Investor's receiving wallet
Investor  ──withdraw_principal► Principal Vault PDA → Investor's receiving wallet
```

### Instructions

| Instruction | Caller | Description |
|---|---|---|
| `initialize` | Owner | Create pool, set goal (50k SOL), min deposit, trading wallet |
| `deposit` | Investor | Deposit SOL → forwarded to trading wallet, stake record created |
| `deposit_profit` | Owner | Deposit batch profit → one tx updates all investor balances |
| `claim_rewards` | Investor | Pull accumulated profit from reward vault |
| `return_principal` | Owner | Fund principal vault for a specific investor |
| `withdraw_principal` | Investor | Withdraw deposit after lock period expires |
| `update_config` | Owner | Change min deposit, trading wallet, open/close pool |

---

## Step-by-Step Deployment

### 1. Install the toolchain

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Solana CLI (v1.18+)
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install 0.30.1
avm use 0.30.1

# Install Node (for tests)
# Use nvm or download from https://nodejs.org
```

### 2. Create your deployer wallet

```bash
# Generate a new keypair (or use your existing one)
solana-keygen new --outfile ~/.config/solana/id.json

# Fund it with SOL for deployment fees
# On devnet:
solana airdrop 2 --url devnet

# Check balance
solana balance --url devnet
```

### 3. Get your program ID

```bash
cd d:\solanabank-contract

# Generate the program keypair
anchor keys list
# Prints something like: trading_pool: AbCdEf...

# IMPORTANT: Copy the printed address into TWO places:
# 1. programs/trading-pool/src/lib.rs  → declare_id!("AbCdEf...")
# 2. Anchor.toml → [programs.devnet] trading_pool = "AbCdEf..."
```

### 4. Build

```bash
anchor build
```

### 5. Deploy to devnet (test first)

```bash
anchor deploy --provider.cluster devnet
```

### 6. Initialize the pool

After deploying, call `initialize` with your parameters.

You can use the Anchor TypeScript client or the admin frontend:

```typescript
import * as anchor from "@coral-xyz/anchor";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.TradingPool;

await program.methods
  .initialize(
    new anchor.web3.PublicKey("7KCJLv8MdEQV62umoqmjkQ6JNzS4oT8fAd7s3y1EYyQP"),
    new anchor.BN(50_000),   // goal: 50,000 SOL
    new anchor.BN(125),      // min deposit: 125 SOL (~$10k at ~$80/SOL)
  )
  .accounts({
    owner: provider.wallet.publicKey,
    // pool, rewardVault, principalVault are PDAs — Anchor resolves them
  })
  .rpc();
```

### 7. Deploy to mainnet

```bash
# Switch to mainnet wallet (make sure it has ~2 SOL for deployment fees)
solana config set --url mainnet-beta

# Deploy
anchor deploy --provider.cluster mainnet
```

---

## Pool Parameters

| Parameter | Value | Notes |
|---|---|---|
| `goal_sol` | 50,000 | Pool closes to new deposits when reached |
| `min_deposit_sol` | 100 SOL (~$10k) | Adjustable via `update_config` |
| `lock_days` options | 730 / 1095 / 1825 | 2yr / 3yr / 5yr — investor chooses at deposit |

---

## Reward Math (Synthetix Pattern)

```
Global accumulator (updated on every profit deposit):
  acc_reward_per_share += profit_lamports × 1e12 / total_deposited

Per investor (computed on claim):
  pending = stake × acc_reward_per_share / 1e12 − reward_debt

After claim:
  reward_debt = stake × acc_reward_per_share / 1e12
```

**Why this works at scale:**
- 1,000 investors → 1 owner tx to distribute profit ($0.0001 fee)
- Each investor claims independently on their own schedule
- Zero owner gas per investor, zero iteration

---

## Security Properties

- **Owner receives deposits immediately** — no escrow delay
- **Investor's stake is immutable on-chain** — cannot be deleted by anyone
- **Reward vault is program-owned** — only the program can move funds out (to investors)
- **Principal vault is program-owned** — same protection
- **No admin key can withdraw from vaults** — the program only allows outflows to the stake's `receiving_wallet`
- **Lock period enforced on-chain** — `withdraw_principal` fails until `lock_end_time` passes
- **Re-entrancy safe** — Anchor account constraints prevent re-entrancy
- **Overflow safe** — all arithmetic uses `checked_*` or `u128`

---

## Project Structure

```
programs/trading-pool/src/
├── lib.rs                          # Program entry point, instruction routing
├── state.rs                        # Pool and Stake account structs
├── errors.rs                       # Custom error codes
└── instructions/
    ├── mod.rs
    ├── initialize.rs               # Owner: create pool
    ├── deposit.rs                  # Investor: deposit SOL
    ├── deposit_profit.rs           # Owner: return trading profits
    ├── claim_rewards.rs            # Investor: pull profit
    ├── return_principal.rs         # Owner: fund principal vault
    ├── withdraw_principal.rs       # Investor: pull principal after lock
    └── update_config.rs            # Owner: change settings
```
