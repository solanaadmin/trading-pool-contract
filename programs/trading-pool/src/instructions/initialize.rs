use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::PoolError;

/// Creates the pool account.  Called once by the deployer.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = Pool::LEN,
        seeds = [POOL_SEED, owner.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    trading_wallet: Pubkey,
    goal_sol: u64,
    min_deposit_sol: u64,
) -> Result<()> {
    require!(goal_sol > 0, PoolError::Overflow);
    require!(min_deposit_sol > 0, PoolError::BelowMinimum);

    let pool = &mut ctx.accounts.pool;

    pool.owner                    = ctx.accounts.owner.key();
    pool.trading_wallet           = trading_wallet;
    pool.goal_lamports            = goal_sol
        .checked_mul(LAMPORTS_PER_SOL)
        .ok_or(PoolError::Overflow)?;
    pool.min_deposit_lamports     = min_deposit_sol
        .checked_mul(LAMPORTS_PER_SOL)
        .ok_or(PoolError::Overflow)?;
    pool.total_deposited          = 0;
    pool.investor_count           = 0;
    pool.acc_reward_per_share     = 0;
    pool.total_profit_deposited   = 0;
    pool.total_principal_returned = 0;
    pool.reward_vault_balance     = 0;
    pool.principal_vault_balance  = 0;
    pool.is_open                  = true;
    pool.bump                     = ctx.bumps.pool;

    msg!(
        "Pool initialized. Goal: {} SOL. Min: {} SOL. Trading wallet: {}",
        goal_sol, min_deposit_sol, trading_wallet,
    );
    Ok(())
}
