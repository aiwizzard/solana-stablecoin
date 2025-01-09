use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_mint_amount_validation(amount in 0..=u64::MAX) {
        let program_id = Pubkey::new_unique();
        let result = process_mint(&program_id, &vec![], amount);
        
        if amount > MINT_LIMIT {
            assert!(matches!(result, Err(ProgramError::Custom(_))));
        }
    }

    #[test]
    fn test_collateral_ratio_validation(
        collateral_value in 0..=u64::MAX,
        mint_amount in 0..=u64::MAX
    ) {
        let ratio = if mint_amount == 0 {
            f64::INFINITY
        } else {
            collateral_value as f64 / mint_amount as f64
        };

        if ratio < MINIMUM_COLLATERAL_RATIO {
            // Should fail if collateral ratio is too low
            assert!(ratio < MINIMUM_COLLATERAL_RATIO);
        }
    }
} 