use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::PoolError;

/// Owner updates pool configuration.
#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        constraint = owner.key() == pool.owner @ PoolError::Unauthorized,
    )]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED, pool.owner.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,
}

pub fn handler(
    ctx: Context<UpdateConfig>,
    new_trading_wallet: Option<Pubkey>,
    new_min_deposit_sol: Option<u64>,
    set_open: Option<bool>,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    if let Some(wallet) = new_trading_wallet {
        pool.trading_wallet = wallet;
        msg!("Trading wallet updated to {}", wallet);
    }

    if let Some(min_sol) = new_min_deposit_sol {
        require!(min_sol > 0, PoolError::BelowMinimum);
        pool.min_deposit_lamports = min_sol
            .checked_mul(LAMPORTS_PER_SOL)
            .ok_or(PoolError::Overflow)?;
        msg!("Min deposit updated to {} SOL", min_sol);
    }

    if let Some(open) = set_open {
        pool.is_open = open;
        msg!("Pool is_open set to {}", open);
    }

    Ok(())
}
