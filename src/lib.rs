use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};
use spl_token::instruction::{burn, mint_to};

mod error;
mod oracle;
use crate::{error::StablecoinError, oracle::{get_oracle_price, OraclePrice}};

// Program ID
solana_program::declare_id!("StbcYYHXFR8nqG7YwvLbKBrpYPE4XbG65gKxGHu4vKP");

// Constants
const ORACLE_STALENESS_THRESHOLD: i64 = 300; // 5 minutes
const PRICE_CONF_PERCENTAGE: f64 = 0.01; // 1% confidence interval
const MAX_PRICE_DEVIATION: f64 = 0.05; // 5%
const MINT_LIMIT: u64 = 1_000_000;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StablecoinConfig {
    pub name: String,
    pub symbol: String,
    pub icon_uri: String,
    pub target_currency: String,
    pub mint_authority: Pubkey,
    pub oracle_pubkey: Pubkey,
    pub total_supply: u64,
    pub collateral_ratio: u64,
    pub last_oracle_price: f64,
    pub last_update_timestamp: i64,
    pub admin_authority: Pubkey,
    pub oracle_authority: Pubkey,
    pub fee_collector: Pubkey,
    pub is_paused: bool,
    pub authorized_minters: Vec<Pubkey>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum StablecoinInstruction {
    Initialize {
        name: String,
        symbol: String,
        icon_uri: String,
        target_currency: String,
    },
    Mint {
        amount: u64,
    },
    Redeem {
        amount: u64,
    },
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = StablecoinInstruction::try_from_slice(instruction_data)?;

    match instruction {
        StablecoinInstruction::Initialize { name, symbol, icon_uri, target_currency } => {
            process_initialize(program_id, accounts, name, symbol, icon_uri, target_currency)
        }
        StablecoinInstruction::Mint { amount } => {
            process_mint(program_id, accounts, amount)
        }
        StablecoinInstruction::Redeem { amount } => {
            process_redeem(program_id, accounts, amount)
        }
    }
}

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    icon_uri: String,
    target_currency: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let config_account = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;

    // Verify the account is owned by our program
    if config_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let config = StablecoinConfig {
        name,
        symbol,
        icon_uri,
        target_currency,
        mint_authority: *mint_authority.key,
        oracle_pubkey: *oracle_account.key,
        total_supply: 0,
        collateral_ratio: 0,
        last_oracle_price: 0.0,
        last_update_timestamp: 0,
        admin_authority: *mint_authority.key,
        oracle_authority: *oracle_account.key,
        fee_collector: *mint_authority.key,
        is_paused: false,
        authorized_minters: vec![*mint_authority.key],
    };

    config.serialize(&mut *config_account.data.borrow_mut())?;
    Ok(())
}

fn validate_price(
    current_price: f64,
    last_price: f64,
    confidence: f64,
) -> ProgramResult {
    // Check price confidence
    let conf_ratio = confidence / current_price.abs();
    if conf_ratio > PRICE_CONF_PERCENTAGE {
        return Err(StablecoinError::PriceUncertain.into());
    }

    // Check price deviation
    if last_price > 0.0 {
        let price_change = (current_price - last_price).abs() / last_price;
        if price_change > MAX_PRICE_DEVIATION {
            return Err(StablecoinError::PriceDeviationTooHigh.into());
        }
    }

    Ok(())
}

fn process_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let config_account = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let user_token_account = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;
    let clock_sysvar = next_account_info(accounts_iter)?;

    let mut config = StablecoinConfig::try_from_slice(&config_account.data.borrow())?;

    // Check program is not paused
    if config.is_paused {
        return Err(StablecoinError::ProgramPaused.into());
    }

    // Verify mint authority
    if !mint_authority.is_signer || !config.authorized_minters.contains(mint_authority.key) {
        return Err(StablecoinError::UnauthorizedMinter.into());
    }

    // Check mint limit
    if amount > MINT_LIMIT {
        return Err(StablecoinError::MintLimitExceeded.into());
    }

    // Get and validate price
    let current_price = get_oracle_price(oracle_account)?;
    let oracle_data = OraclePrice::try_from_slice(&oracle_account.data.borrow())?;
    
    validate_price(
        current_price,
        config.last_oracle_price,
        oracle_data.confidence,
    )?;

    // Check oracle staleness
    let clock = Clock::from_account_info(clock_sysvar)?;
    if clock.unix_timestamp - oracle_data.last_update_timestamp > ORACLE_STALENESS_THRESHOLD {
        return Err(StablecoinError::StaleOracleData.into());
    }

    // Calculate tokens to mint based on price
    let tokens_to_mint = (amount as f64 / current_price) as u64;

    // Mint tokens to user account
    let mint_ix = mint_to(
        &spl_token::id(),
        token_mint.key,
        user_token_account.key,
        mint_authority.key,
        &[],
        tokens_to_mint,
    )?;

    solana_program::program::invoke_signed(
        &mint_ix,
        &[
            token_mint.clone(),
            user_token_account.clone(),
            mint_authority.clone(),
        ],
        &[],
    )?;

    // Update config state
    config.total_supply = config.total_supply.checked_add(tokens_to_mint)
        .ok_or(StablecoinError::InvalidAmount)?;
    config.last_oracle_price = current_price;
    config.last_update_timestamp = clock.unix_timestamp;
    config.serialize(&mut *config_account.data.borrow_mut())?;

    msg!("Minted {} tokens", tokens_to_mint);
    Ok(())
}

fn process_redeem(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let config_account = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let user_token_account = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;
    let clock_sysvar = next_account_info(accounts_iter)?;

    let mut config = StablecoinConfig::try_from_slice(&config_account.data.borrow())?;

    // Check program is not paused
    if config.is_paused {
        return Err(StablecoinError::ProgramPaused.into());
    }

    // Verify user is signer
    if !user.is_signer {
        return Err(StablecoinError::Unauthorized.into());
    }

    // Get and validate price
    let current_price = get_oracle_price(oracle_account)?;
    let oracle_data = OraclePrice::try_from_slice(&oracle_account.data.borrow())?;
    
    validate_price(
        current_price,
        config.last_oracle_price,
        oracle_data.confidence,
    )?;

    // Check oracle staleness
    let clock = Clock::from_account_info(clock_sysvar)?;
    if clock.unix_timestamp - oracle_data.last_update_timestamp > ORACLE_STALENESS_THRESHOLD {
        return Err(StablecoinError::StaleOracleData.into());
    }

    // Calculate redemption amount
    let redemption_amount = (amount as f64 * current_price) as u64;

    // Burn tokens
    let burn_ix = burn(
        &spl_token::id(),
        user_token_account.key,
        token_mint.key,
        user.key,
        &[],
        amount,
    )?;

    solana_program::program::invoke(
        &burn_ix,
        &[
            user_token_account.clone(),
            token_mint.clone(),
            user.clone(),
        ],
    )?;

    // Update config state
    config.total_supply = config.total_supply.checked_sub(amount)
        .ok_or(StablecoinError::InvalidAmount)?;
    config.last_oracle_price = current_price;
    config.last_update_timestamp = clock.unix_timestamp;
    config.serialize(&mut *config_account.data.borrow_mut())?;

    msg!("Redeemed {} tokens for {} units of fiat", amount, redemption_amount);
    Ok(())
} 