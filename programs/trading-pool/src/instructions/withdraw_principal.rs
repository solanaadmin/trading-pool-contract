use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::PoolError;

/// Investor withdraws their principal after the lock period expires.
/// Stake account is closed and rent returned to the investor.
#[derive(Accounts)]
pub struct WithdrawPrincipal<'info> {
    pub investor: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED, pool.owner.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        close = investor,
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

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawPrincipal>) -> Result<()> {
    let clock           = Clock::get()?;
    let is_active       = ctx.accounts.stake.is_active;
    let lock_end_time   = ctx.accounts.stake.lock_end_time;
    let amount          = ctx.accounts.stake.amount_lamports;
    let receiving_key   = ctx.accounts.receiving_wallet.key();

    require!(is_active, PoolError::StakeInactive);
    require!(clock.unix_timestamp >= lock_end_time, PoolError::StillLocked);
    require!(
        ctx.accounts.pool.principal_vault_balance >= amount,
        PoolError::InsufficientPrincipalVault,
    );

    // Transfer: Pool PDA → receiving wallet
    **ctx.accounts.pool
        .to_account_info()
        .try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.receiving_wallet
        .to_account_info()
        .try_borrow_mut_lamports()? += amount;

    let pool = &mut ctx.accounts.pool;
    pool.principal_vault_balance = pool.principal_vault_balance
        .checked_sub(amount)
        .ok_or(PoolError::Overflow)?;
    pool.total_deposited = pool.total_deposited
        .checked_sub(amount)
        .ok_or(PoolError::Overflow)?;
    pool.investor_count = pool.investor_count.saturating_sub(1);

    msg!("Principal withdrawn: {:.4} SOL → {}", amount as f64 / 1e9, receiving_key);
    Ok(())
}
