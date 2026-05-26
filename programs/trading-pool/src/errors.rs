use anchor_lang::prelude::*;

#[error_code]
pub enum PoolError {
    #[msg("Pool is not accepting new deposits (goal reached or paused by owner)")]
    PoolClosed,

    #[msg("Deposit amount is below the minimum required")]
    BelowMinimum,

    #[msg("Deposit would exceed the pool's collection goal")]
    ExceedsGoal,

    #[msg("Lock period must be 730 (2yr), 1095 (3yr), or 1825 (5yr) days — minimum 2 years")]
    InvalidLockPeriod,

    #[msg("Principal is still locked — lock period has not expired yet")]
    StillLocked,

    #[msg("Principal has already been withdrawn")]
    AlreadyWithdrawn,

    #[msg("Stake is not active")]
    StakeInactive,

    #[msg("No rewards available to claim")]
    NoRewardsToClaim,

    #[msg("Insufficient funds in the principal vault for this withdrawal")]
    InsufficientPrincipalVault,

    #[msg("Insufficient funds in the reward vault for this claim")]
    InsufficientRewardVault,

    #[msg("Arithmetic overflow")]
    Overflow,

    #[msg("Only the pool owner can call this instruction")]
    Unauthorized,

    #[msg("Profit deposit amount must be greater than zero")]
    ZeroProfitDeposit,

    #[msg("Total deposited is zero — cannot distribute rewards with no active stakes")]
    NoActiveStakes,

    #[msg("Receiving wallet must be provided")]
    MissingReceivingWallet,
}
