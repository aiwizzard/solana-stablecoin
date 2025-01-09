use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
};
use borsh::{BorshDeserialize, BorshSerialize};
use crate::error::StablecoinError;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct OraclePrice {
    pub price: f64,
    pub confidence: f64,
    pub last_update_timestamp: i64,
}

pub fn get_oracle_price(oracle_account: &AccountInfo) -> Result<f64, ProgramError> {
    let oracle_data = OraclePrice::try_from_slice(&oracle_account.data.borrow())
        .map_err(|_| StablecoinError::InvalidOracleData)?;
    
    Ok(oracle_data.price)
} 