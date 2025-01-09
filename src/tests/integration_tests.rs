use super::*;

#[tokio::test]
async fn test_full_stablecoin_flow() {
    let mut program_test = ProgramTest::new(
        "solana_stablecoin",
        crate::id(),
        processor!(crate::process_instruction),
    );

    let mut context = TestContext::new().await;
    let mut banks_client = program_test.start_with_context().await;

    // Initialize program
    context.initialize(&mut banks_client).await.unwrap();

    // Test initialization
    let init_ix = StablecoinInstruction::Initialize {
        name: "Test Coin".to_string(),
        symbol: "TEST".to_string(),
        icon_uri: "http://test.com/icon".to_string(),
        target_currency: "USD".to_string(),
    };

    let transaction = Transaction::new_signed_with_payer(
        &[instruction::initialize(
            &context.program_id,
            &context.config,
            &context.mint_authority.pubkey(),
            &context.oracle,
            &init_ix,
        )],
        Some(&context.admin.pubkey()),
        &[&context.admin],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Test minting
    let mint_amount = 1000;
    let mint_ix = StablecoinInstruction::Mint {
        amount: mint_amount,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[instruction::mint(
            &context.program_id,
            &context.config,
            &context.token_mint,
            &context.user.pubkey(),
            &context.mint_authority.pubkey(),
            &mint_ix,
        )],
        Some(&context.mint_authority.pubkey()),
        &[&context.mint_authority],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Test redeeming
    let redeem_amount = 500;
    let redeem_ix = StablecoinInstruction::Redeem {
        amount: redeem_amount,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[instruction::redeem(
            &context.program_id,
            &context.config,
            &context.token_mint,
            &context.user.pubkey(),
            &redeem_ix,
        )],
        Some(&context.user.pubkey()),
        &[&context.user],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Verify final state
    let config_account = banks_client
        .get_account(context.config)
        .await
        .unwrap()
        .unwrap();
    
    let config = StablecoinConfig::try_from_slice(&config_account.data).unwrap();
    assert_eq!(config.total_supply, mint_amount - redeem_amount);
}

#[tokio::test]
async fn test_authority_management() {
    let mut program_test = ProgramTest::new(
        "solana_stablecoin",
        crate::id(),
        processor!(crate::process_instruction),
    );

    let mut context = TestContext::new().await;
    let mut banks_client = program_test.start_with_context().await;

    // Initialize program
    context.initialize(&mut banks_client).await.unwrap();

    // Test adding a new minter
    let new_minter = Keypair::new();
    let add_minter_ix = StablecoinInstruction::AddMinter {
        minter: new_minter.pubkey(),
    };

    let transaction = Transaction::new_signed_with_payer(
        &[instruction::add_minter(
            &context.program_id,
            &context.config,
            &context.admin.pubkey(),
            &add_minter_ix,
        )],
        Some(&context.admin.pubkey()),
        &[&context.admin],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Verify minter was added
    let config_account = banks_client
        .get_account(context.config)
        .await
        .unwrap()
        .unwrap();
    
    let config = StablecoinConfig::try_from_slice(&config_account.data).unwrap();
    assert!(config.authorized_minters.contains(&new_minter.pubkey()));
} 