use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
#[allow(unused_imports)]

use crate::state::*;
use crate::errors::PoolError;

/// Minimum lock duration: 2 years (730 days).
/// Accepted values: 730 (2yr), 1095 (3yr), 1825 (5yr).
/// The contract enforces the minimum; the frontend should present these options.
const MIN_LOCK_DAYS: u16 = 730;
const VALID_LOCK_DAYS: [u16; 3] = [730, 1095, 1825];

/// Investor deposits SOL into the fund.
///
/// Flow:
///   1. Validates pool is open, amount ≥ min, deposit doesn't exceed goal.
///   2. Transfers SOL from investor → pool owner's trading wallet (directly).
///   3. Creates a `Stake` PDA recording the obligation on-chain.
///   4. Sets `reward_debt` so the investor cannot claim profit accumulated
///      before their deposit.
#[derive(Accounts)]
pub struct Deposit<'info> {
    /// The investor signing the deposit.
    #[account(mut)]
    pub investor: Signer<'info>,

    /// Pool global state.
    #[account(
        mut,
        seeds = [POOL_SEED, pool.owner.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    /// The trading wallet that receives the deposit.
    /// Must match pool.trading_wallet.
    #[account(
        mut,
        constraint = trading_wallet.key() == pool.trading_wallet
            @ PoolError::Unauthorized,
    )]
    pub trading_wallet: SystemAccount<'info>,

    /// Stake record for this (pool, investor) pair.
    /// Created fresh — each investor can only have one active stake per pool.
    #[account(
        init,
        payer = investor,
        space = Stake::LEN,
        seeds = [STAKE_SEED, pool.key().as_ref(), investor.key().as_ref()],
        bump,
    )]
    pub stake: Account<'info, Stake>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Deposit>,
    amount_lamports: u64,
    lock_days: u16,
    receiving_wallet: Pubkey,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // ── Validations ────────────────────────────────────────────────────
    require!(pool.is_open, PoolError::PoolClosed);
    require!(amount_lamports >= pool.min_deposit_lamports, PoolError::BelowMinimum);
    require!(
        lock_days >= MIN_LOCK_DAYS && VALID_LOCK_DAYS.contains(&lock_days),
        PoolError::InvalidLockPeriod,
    );

    let new_total = pool.total_deposited
        .checked_add(amount_lamports)
        .ok_or(PoolError::Overflow)?;
    require!(new_total <= pool.goal_lamports, PoolError::ExceedsGoal);

    // ── Transfer SOL: investor → trading wallet ────────────────────────
    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.investor.to_account_info(),
            to:   ctx.accounts.trading_wallet.to_account_info(),
        },
    );
    system_program::transfer(cpi_ctx, amount_lamports)?;

    // ── Record the stake ───────────────────────────────────────────────
    let clock = Clock::get()?;
    let lock_seconds = (lock_days as i64)
        .checked_mul(86_400)
        .ok_or(PoolError::Overflow)?;

    let stake = &mut ctx.accounts.stake;
    stake.pool              = pool.key();
    stake.investor          = ctx.accounts.investor.key();
    stake.amount_lamports   = amount_lamports;
    stake.deposit_time      = clock.unix_timestamp;
    stake.lock_end_time     = clock.unix_timestamp
        .checked_add(lock_seconds)
        .ok_or(PoolError::Overflow)?;
    stake.lock_days         = lock_days;
    stake.is_active         = true;
    stake.receiving_wallet  = receiving_wallet;
    stake.total_claimed_rewards = 0;
    stake.bump              = ctx.bumps.stake;

    // Debt snapshot — prevents claiming rewards accumulated before this deposit.
    stake.reward_debt = (amount_lamports as u128)
        .checked_mul(pool.acc_reward_per_share)
        .ok_or(PoolError::Overflow)?
        .checked_div(REWARD_PRECISION)
        .ok_or(PoolError::Overflow)?;

    // ── Update pool totals ─────────────────────────────────────────────
    pool.total_deposited = new_total;
    pool.investor_count  = pool.investor_count
        .checked_add(1)
        .ok_or(PoolError::Overflow)?;

    // Auto-close pool when goal is reached.
    if pool.total_deposited >= pool.goal_lamports {
        pool.is_open = false;
        msg!("Pool goal reached — closed to new deposits.");
    }

    msg!(
        "Deposit recorded: {} lamports, {} day lock. Investor: {}. Receiving: {}",
        amount_lamports,
        lock_days,
        ctx.accounts.investor.key(),
        receiving_wallet,
    );

    Ok(())
}
