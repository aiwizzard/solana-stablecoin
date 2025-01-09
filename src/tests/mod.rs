use super::*;
use solana_program::{
    account_info::AccountInfo,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

mod unit_tests;
mod integration_tests;

// Test helper functions
pub struct TestContext {
    pub program_id: Pubkey,
    pub admin: Keypair,
    pub mint_authority: Keypair,
    pub oracle_authority: Keypair,
    pub user: Keypair,
    pub config: Pubkey,
    pub token_mint: Pubkey,
    pub oracle: Pubkey,
}

impl TestContext {
    pub async fn new() -> Self {
        let program_id = Pubkey::new_unique();
        let admin = Keypair::new();
        let mint_authority = Keypair::new();
        let oracle_authority = Keypair::new();
        let user = Keypair::new();
        let config = Pubkey::new_unique();
        let token_mint = Pubkey::new_unique();
        let oracle = Pubkey::new_unique();

        Self {
            program_id,
            admin,
            mint_authority,
            oracle_authority,
            user,
            config,
            token_mint,
            oracle,
        }
    }

    pub async fn initialize(&self, banks_client: &mut BanksClient) -> Result<(), BanksClientError> {
        let rent = banks_client.get_rent().await?;
        let config_size = 1000; // Adjust based on actual size needed

        let ix = system_instruction::create_account(
            &self.admin.pubkey(),
            &self.config,
            rent.minimum_balance(config_size),
            config_size as u64,
            &self.program_id,
        );

        let transaction = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.admin.pubkey()),
            &[&self.admin],
            banks_client.get_latest_blockhash().await?,
        );

        banks_client.process_transaction(transaction).await
    }
} 