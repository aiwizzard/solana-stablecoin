use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum StablecoinError {
    #[error("Invalid Oracle Data")]
    InvalidOracleData,
    #[error("Invalid Price Feed")]
    InvalidPriceFeed,
    #[error("Insufficient Collateral")]
    InsufficientCollateral,
    #[error("Invalid Amount")]
    InvalidAmount,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Stale Oracle Data")]
    StaleOracleData,
    #[error("Price Deviation Too High")]
    PriceDeviationTooHigh,
    #[error("Mint Limit Exceeded")]
    MintLimitExceeded,
    #[error("Invalid Token Account")]
    InvalidTokenAccount,
    #[error("Invalid Mint")]
    InvalidMint,
    #[error("Invalid Liquidation")]
    InvalidLiquidation,
    #[error("Below Redemption Price")]
    BelowRedemptionPrice,
    #[error("Above Liquidation Threshold")]
    AboveLiquidationThreshold,
    #[error("Invalid Parameter Adjustment")]
    InvalidParameterAdjustment,
    #[error("Insufficient Stability Fees")]
    InsufficientStabilityFees,
    #[error("Program is paused")]
    ProgramPaused,
    #[error("Unauthorized minter")]
    UnauthorizedMinter,
    #[error("Invalid authority type")]
    InvalidAuthorityType,
    #[error("Authority already exists")]
    AuthorityAlreadyExists,
    #[error("Maximum minters reached")]
    MaxMintersReached,
    #[error("Insufficient signatures")]
    InsufficientSignatures,
    #[error("Timelock not expired")]
    TimelockNotExpired,
    #[error("Unauthorized delegate")]
    UnauthorizedDelegate,
    #[error("Invalid delegation")]
    InvalidDelegation,
    #[error("Delegation expired")]
    DelegationExpired,
    #[error("Price confidence interval too large")]
    PriceUncertain,
    #[error("Invalid oracle price feed")]
    InvalidOraclePriceFeed,
}

impl From<StablecoinError> for ProgramError {
    fn from(e: StablecoinError) -> Self {
        ProgramError::Custom(e as u32)
    }
} 