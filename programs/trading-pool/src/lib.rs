use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

// ── Program ID ─────────────────────────────────────────────────────────────
//
// Replace this placeholder after running:
//   anchor keys list
// then paste the printed key here and in Anchor.toml [programs.*]
declare_id!("H59K28q5kyteWfAGqt34pwkduNwntcambE75tXVp3nZF");

#[program]
pub mod trading_pool {
    use super::*;

    // ── Admin instructions (owner only) ────────────────────────────────────

    /// Create the pool.  Called once by the deployer.
    ///
    /// Parameters:
    ///   - `trading_wallet`   — Solana pubkey that receives investor deposits
    ///   - `goal_sol`         — Total SOL cap (e.g. 50_000)
    ///   - `min_deposit_sol`  — Per-investor minimum in whole SOL (e.g. 100)
    pub fn initialize(
        ctx: Context<Initialize>,
        trading_wallet: Pubkey,
        goal_sol: u64,
        min_deposit_sol: u64,
    ) -> Result<()> {
        initialize::handler(ctx, trading_wallet, goal_sol, min_deposit_sol)
    }

    /// Deposit trading profits into the reward vault.
    ///
    /// One transaction updates every investor's claimable balance.
    /// The Synthetix accumulator pattern ensures O(1) cost regardless
    /// of the number of investors.
    ///
    /// Parameters:
    ///   - `amount_lamports` — Profit amount in lamports
    pub fn deposit_profit(
        ctx: Context<DepositProfit>,
        amount_lamports: u64,
    ) -> Result<()> {
        deposit_profit::handler(ctx, amount_lamports)
    }

    /// Return an investor's principal into the principal vault.
    ///
    /// Called by the owner once for each investor whose lock period is
    /// approaching expiry.  The investor can then call `withdraw_principal`.
    pub fn return_principal(ctx: Context<ReturnPrincipal>) -> Result<()> {
        return_principal::handler(ctx)
    }

    /// Update pool configuration (trading wallet, min deposit, open/close).
    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_trading_wallet: Option<Pubkey>,
        new_min_deposit_sol: Option<u64>,
        set_open: Option<bool>,
    ) -> Result<()> {
        update_config::handler(ctx, new_trading_wallet, new_min_deposit_sol, set_open)
    }

    // ── Investor instructions (permissionless) ─────────────────────────────

    /// Investor deposits SOL into the fund.
    ///
    /// SOL is transferred directly to the owner's trading wallet.
    /// An on-chain stake record is created as proof of the obligation.
    ///
    /// Parameters:
    ///   - `amount_lamports`   — Deposit amount in lamports
    ///   - `lock_days`         — 730 (2yr), 1095 (3yr), or 1825 (5yr)
    ///   - `receiving_wallet`  — Address where rewards and principal are sent
    pub fn deposit(
        ctx: Context<Deposit>,
        amount_lamports: u64,
        lock_days: u16,
        receiving_wallet: Pubkey,
    ) -> Result<()> {
        deposit::handler(ctx, amount_lamports, lock_days, receiving_wallet)
    }

    /// Claim accumulated trading profit.
    ///
    /// The investor calls this at any time after profit has been deposited.
    /// SOL is transferred from the reward vault to the investor's
    /// receiving_wallet.  No approval from the owner is required.
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        claim_rewards::handler(ctx)
    }

    /// Withdraw original principal after the lock period expires.
    ///
    /// The owner must have called `return_principal` first to fund the
    /// principal vault.  After withdrawal, the stake account is closed
    /// and the rent lamports are returned to the investor.
    pub fn withdraw_principal(ctx: Context<WithdrawPrincipal>) -> Result<()> {
        withdraw_principal::handler(ctx)
    }
}
