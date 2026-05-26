use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use crate::state::*;
use crate::errors::PoolError;

/// Owner deposits trading profit into the Pool PDA (acts as reward vault).
/// One transaction — all investor balances update via the accumulator.
#[derive(Accounts)]
pub struct DepositProfit<'info> {
    #[account(
        mut,
        constraint = owner.key() == pool.owner @ PoolError::Unauthorized,
    )]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED, pool.owner.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DepositProfit>, amount_lamports: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    require!(amount_lamports > 0, PoolError::ZeroProfitDeposit);
    require!(pool.total_deposited > 0, PoolError::NoActiveStakes);

    // Read total_deposited before mutable borrow
    let total_deposited = pool.total_deposited;
    drop(pool);

    // Transfer profit SOL: owner → Pool PDA
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.owner.to_account_info(),
                to:   ctx.accounts.pool.to_account_info(),
            },
        ),
        amount_lamports,
    )?;

    let pool = &mut ctx.accounts.pool;

    // Update Synthetix accumulator
    let increment = (amount_lamports as u128)
        .checked_mul(REWARD_PRECISION)
        .ok_or(PoolError::Overflow)?
        .checked_div(total_deposited as u128)
        .ok_or(PoolError::Overflow)?;

    pool.acc_reward_per_share = pool.acc_reward_per_share
        .checked_add(increment)
        .ok_or(PoolError::Overflow)?;

    pool.total_profit_deposited = pool.total_profit_deposited
        .checked_add(amount_lamports)
        .ok_or(PoolError::Overflow)?;

    pool.reward_vault_balance = pool.reward_vault_balance
        .checked_add(amount_lamports)
        .ok_or(PoolError::Overflow)?;

    msg!(
        "Profit deposited: {:.4} SOL. Total returned: {:.4} SOL.",
        amount_lamports as f64 / 1e9,
        pool.total_profit_deposited as f64 / 1e9,
    );
    Ok(())
}
