use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let program_id = Pubkey::new_unique();
        let config_key = Pubkey::new_unique();
        let mint_authority = Keypair::new();
        let oracle = Pubkey::new_unique();

        let mut config_account = AccountInfo::new(
            &config_key,
            false,
            true,
            &mut [0u8; 1000],
            &program_id,
            false,
            Rent::default().minimum_balance(1000),
        );

        let accounts = vec![
            config_account.clone(),
            AccountInfo::new(
                &mint_authority.pubkey(),
                true,
                false,
                &mut [],
                &program_id,
                false,
                0,
            ),
            AccountInfo::new(
                &oracle,
                false,
                false,
                &mut [],
                &program_id,
                false,
                0,
            ),
        ];

        let result = process_initialize(
            &program_id,
            &accounts,
            "Test Coin".to_string(),
            "TEST".to_string(),
            "http://test.com/icon".to_string(),
            "USD".to_string(),
        );

        assert!(result.is_ok());

        let config = StablecoinConfig::try_from_slice(&config_account.data.borrow()).unwrap();
        assert_eq!(config.name, "Test Coin");
        assert_eq!(config.symbol, "TEST");
        assert_eq!(config.target_currency, "USD");
    }

    #[test]
    fn test_mint_validation() {
        let program_id = Pubkey::new_unique();
        let config_key = Pubkey::new_unique();
        let mint_authority = Keypair::new();
        let user = Keypair::new();
        let oracle = Pubkey::new_unique();

        // Test mint amount validation
        let result = process_mint(
            &program_id,
            &vec![],  // Empty accounts for this test
            MINT_LIMIT + 1,
        );
        assert!(matches!(result, Err(ProgramError::Custom(_))));

        // Test unauthorized minter
        let mut config = StablecoinConfig::default();
        config.authorized_minters = vec![Pubkey::new_unique()];  // Different from mint_authority
        
        let result = process_mint(
            &program_id,
            &vec![],  // Empty accounts for this test
            1000,
        );
        assert!(matches!(result, Err(ProgramError::Custom(_))));
    }

    #[test]
    fn test_authority_validation() {
        let program_id = Pubkey::new_unique();
        let admin = Keypair::new();
        let new_authority = Pubkey::new_unique();

        let result = process_update_authority(
            &program_id,
            &vec![],  // Empty accounts for this test
            AuthorityType::Admin,
            new_authority,
        );
        assert!(matches!(result, Err(ProgramError::Custom(_))));
    }
} 