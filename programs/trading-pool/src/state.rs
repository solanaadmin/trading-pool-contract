use anchor_lang::prelude::*;

pub const REWARD_PRECISION: u128 = 1_000_000_000_000;
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub const POOL_SEED: &[u8] = b"pool";
pub const STAKE_SEED: &[u8] = b"stake";

/// Global pool state — one per deployment.
/// The Pool PDA also acts as the single SOL vault for both rewards
/// and principal returns (tracked via separate bookkeeping fields).
#[account]
#[derive(Default)]
pub struct Pool {
    pub owner: Pubkey,
    pub trading_wallet: Pubkey,
    pub goal_lamports: u64,
    pub min_deposit_lamports: u64,
    pub total_deposited: u64,
    pub investor_count: u32,
    /// Synthetix accumulator: acc_reward_per_share += profit * PRECISION / total_deposited
    pub acc_reward_per_share: u128,
    pub total_profit_deposited: u64,
    pub total_principal_returned: u64,
    /// Tracks how much SOL in the pool is earmarked for reward claims.
    pub reward_vault_balance: u64,
    /// Tracks how much SOL in the pool is earmarked for principal withdrawals.
    pub principal_vault_balance: u64,
    pub is_open: bool,
    pub bump: u8,
}

impl Pool {
    pub const LEN: usize = 8
        + 32  // owner
        + 32  // trading_wallet
        + 8   // goal_lamports
        + 8   // min_deposit_lamports
        + 8   // total_deposited
        + 4   // investor_count
        + 16  // acc_reward_per_share
        + 8   // total_profit_deposited
        + 8   // total_principal_returned
        + 8   // reward_vault_balance
        + 8   // principal_vault_balance
        + 1   // is_open
        + 1;  // bump
}

/// Per-investor stake record.
#[account]
pub struct Stake {
    pub pool: Pubkey,
    pub investor: Pubkey,
    pub amount_lamports: u64,
    pub deposit_time: i64,
    pub lock_end_time: i64,
    pub lock_days: u16,
    pub reward_debt: u128,
    pub total_claimed_rewards: u64,
    pub is_active: bool,
    pub receiving_wallet: Pubkey,
    pub bump: u8,
}

impl Stake {
    pub const LEN: usize = 8
        + 32  // pool
        + 32  // investor
        + 8   // amount_lamports
        + 8   // deposit_time
        + 8   // lock_end_time
        + 2   // lock_days
        + 16  // reward_debt
        + 8   // total_claimed_rewards
        + 1   // is_active
        + 32  // receiving_wallet
        + 1;  // bump
}
