use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use crate::state::*;
use crate::errors::PoolError;

/// Owner returns an investor's principal into the Pool PDA.
/// After this, the investor can call withdraw_principal.
#[derive(Accounts)]
pub struct ReturnPrincipal<'info> {
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

    #[account(
        seeds = [STAKE_SEED, pool.key().as_ref(), stake.investor.as_ref()],
        bump = stake.bump,
    )]
    pub stake: Account<'info, Stake>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ReturnPrincipal>) -> Result<()> {
    require!(ctx.accounts.stake.is_active, PoolError::StakeInactive);

    let amount = ctx.accounts.stake.amount_lamports;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.owner.to_account_info(),
                to:   ctx.accounts.pool.to_account_info(),
            },
        ),
        amount,
    )?;

    let pool = &mut ctx.accounts.pool;
    pool.total_principal_returned = pool.total_principal_returned
        .checked_add(amount)
        .ok_or(PoolError::Overflow)?;
    pool.principal_vault_balance = pool.principal_vault_balance
        .checked_add(amount)
        .ok_or(PoolError::Overflow)?;

    msg!(
        "Principal returned for {}: {:.4} SOL",
        ctx.accounts.stake.investor,
        amount as f64 / 1e9,
    );
    Ok(())
}
