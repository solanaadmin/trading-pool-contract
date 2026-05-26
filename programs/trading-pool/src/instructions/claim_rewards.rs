use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::PoolError;

/// Investor pulls their accumulated trading profit from the Pool PDA.
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    pub investor: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED, pool.owner.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [STAKE_SEED, pool.key().as_ref(), investor.key().as_ref()],
        bump = stake.bump,
        constraint = stake.investor == investor.key() @ PoolError::Unauthorized,
    )]
    pub stake: Account<'info, Stake>,

    /// CHECK: payout destination — validated against stake.receiving_wallet
    #[account(
        mut,
        constraint = receiving_wallet.key() == stake.receiving_wallet @ PoolError::Unauthorized,
    )]
    pub receiving_wallet: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<ClaimRewards>) -> Result<()> {
    let stake  = &ctx.accounts.stake;
    let pool   = &ctx.accounts.pool;

    require!(stake.is_active, PoolError::StakeInactive);

    let gross = (stake.amount_lamports as u128)
        .checked_mul(pool.acc_reward_per_share)
        .ok_or(PoolError::Overflow)?
        .checked_div(REWARD_PRECISION)
        .ok_or(PoolError::Overflow)?;

    let pending = gross
        .checked_sub(stake.reward_debt)
        .ok_or(PoolError::Overflow)?;

    require!(pending > 0, PoolError::NoRewardsToClaim);
    let pending_lamports = pending as u64;

    require!(
        pool.reward_vault_balance >= pending_lamports,
        PoolError::InsufficientRewardVault,
    );

    // Transfer: Pool PDA → receiving_wallet (direct lamport manipulation)
    **ctx.accounts.pool
        .to_account_info()
        .try_borrow_mut_lamports()? -= pending_lamports;
    **ctx.accounts.receiving_wallet
        .to_account_info()
        .try_borrow_mut_lamports()? += pending_lamports;

    // Update stake and pool state
    let stake  = &mut ctx.accounts.stake;
    stake.reward_debt = gross;
    stake.total_claimed_rewards = stake.total_claimed_rewards
        .checked_add(pending_lamports)
        .ok_or(PoolError::Overflow)?;

    let pool = &mut ctx.accounts.pool;
    pool.reward_vault_balance = pool.reward_vault_balance
        .checked_sub(pending_lamports)
        .ok_or(PoolError::Overflow)?;

    msg!(
        "Reward claimed: {:.6} SOL → {}",
        pending_lamports as f64 / 1e9,
        ctx.accounts.receiving_wallet.key(),
    );
    Ok(())
}
