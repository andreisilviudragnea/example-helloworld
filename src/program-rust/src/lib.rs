use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    _program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    _instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    msg!(
        "Hello World Rust program entrypoint {}",
        accounts[1].data.borrow()[0]
    );

    Ok(())
}

// Sanity tests
#[cfg(test)]
mod test {
    use solana_program::bpf_loader_upgradeable::UpgradeableLoaderState;
    use solana_program::clock::Slot;
    use solana_program::instruction::{AccountMeta, Instruction};
    use solana_program::rent::Rent;
    use solana_program::{bpf_loader, bpf_loader_upgradeable};
    use solana_program_test::{read_file, tokio, ProgramTest, ProgramTestContext};
    use solana_sdk::account::{Account, AccountSharedData};
    use solana_sdk::account_utils::StateMut;
    use solana_sdk::signature::Signer;
    use solana_sdk::transaction::Transaction;

    use super::*;

    #[tokio::test]
    async fn test_set_non_upgradeable_program_account_does_not_work() {
        let program_id = Pubkey::new_unique();

        let mut context = ProgramTest::default().start_with_context().await;

        set_non_upgradeable_program_account(&mut context, program_id, "helloworld0.so");

        let result = simulate_transaction(&mut context, program_id).await;
        assert_eq!(
            result.simulation_details.unwrap().logs[1],
            "Program log: Hello World Rust program entrypoint 0"
        );

        set_non_upgradeable_program_account(&mut context, program_id, "helloworld1.so");

        context.warp_to_slot(2).unwrap();

        let result = simulate_transaction(&mut context, program_id).await;
        assert_eq!(
            result.simulation_details.unwrap().logs[1],
            "Program log: Hello World Rust program entrypoint 0" // TODO should be 1
        );
    }

    fn set_non_upgradeable_program_account(
        context: &mut ProgramTestContext,
        program_id: Pubkey,
        path: &str,
    ) {
        let program_data = read_file(path);

        context.set_account(
            &program_id,
            &AccountSharedData::from(Account {
                lamports: Rent::default().minimum_balance(program_data.len()).max(1),
                data: program_data,
                owner: bpf_loader::id(),
                executable: true,
                rent_epoch: 0,
            }),
        );
    }

    #[tokio::test]
    async fn test_upgradeable_program_account_set_program_data_account_data_works() {
        let program_id = Pubkey::new_unique();

        let mut context = ProgramTest::default().start_with_context().await;

        let program_data_address = Pubkey::new_unique();
        context.set_account(
            &program_id,
            &upgradeable_program_account(program_data_address),
        );

        context.set_account(
            &program_data_address,
            &program_data_account("helloworld0.so", 0),
        );

        let result = simulate_transaction(&mut context, program_id).await;
        assert_eq!(
            result.simulation_details.unwrap().logs[1],
            "Program log: Hello World Rust program entrypoint 0"
        );

        context.set_account(
            &program_data_address,
            &program_data_account("helloworld1.so", 1),
        );

        context.warp_to_slot(2).unwrap();

        let result = simulate_transaction(&mut context, program_id).await;
        assert_eq!(
            result.simulation_details.unwrap().logs[1],
            "Program log: Hello World Rust program entrypoint 1"
        );
    }

    #[tokio::test]
    async fn test_upgradeable_program_account_set_program_data_account_address_works() {
        let program_id = Pubkey::new_unique();

        let mut context = ProgramTest::default().start_with_context().await;

        let program_data_address = Pubkey::new_unique();
        context.set_account(
            &program_id,
            &upgradeable_program_account(program_data_address),
        );

        context.set_account(
            &program_data_address,
            &program_data_account("helloworld1.so", 0),
        );

        let result = simulate_transaction(&mut context, program_id).await;
        assert_eq!(
            result.simulation_details.unwrap().logs[1],
            "Program log: Hello World Rust program entrypoint 1"
        );

        context.warp_to_slot(2).unwrap();

        let program_data_address = Pubkey::new_unique();
        context.set_account(
            &program_id,
            &upgradeable_program_account(program_data_address),
        );
        context.set_account(
            &program_data_address,
            &program_data_account("helloworld0.so", 2),
        );

        context.warp_to_slot(3).unwrap();

        let result = simulate_transaction(&mut context, program_id).await;
        assert_eq!(
            result.simulation_details.unwrap().logs[1],
            "Program log: Hello World Rust program entrypoint 0"
        );
    }

    #[tokio::test]
    async fn test_set_non_program_account_works() {
        let program_id = Pubkey::new_unique();

        let mut context = ProgramTest::default().start_with_context().await;

        let program_data_address = Pubkey::new_unique();
        context.set_account(
            &program_id,
            &upgradeable_program_account(program_data_address),
        );
        context.set_account(
            &program_data_address,
            &program_data_account("helloworld.so", 0),
        );

        let account_address = Pubkey::new_unique();

        context.set_account(
            &account_address,
            &AccountSharedData::from(Account {
                lamports: Rent::default().minimum_balance(1).max(1),
                data: vec![123],
                owner: bpf_loader_upgradeable::id(),
                executable: true,
                rent_epoch: 0,
            }),
        );

        let result =
            simulate_transaction_with_account(&mut context, program_id, account_address).await;
        assert_eq!(
            result.simulation_details.unwrap().logs[1],
            "Program log: Hello World Rust program entrypoint 123"
        );

        context.set_account(
            &account_address,
            &AccountSharedData::from(Account {
                lamports: Rent::default().minimum_balance(1).max(1),
                data: vec![234],
                owner: bpf_loader_upgradeable::id(),
                executable: true,
                rent_epoch: 0,
            }),
        );

        let result =
            simulate_transaction_with_account(&mut context, program_id, account_address).await;
        assert_eq!(
            result.simulation_details.unwrap().logs[1],
            "Program log: Hello World Rust program entrypoint 234"
        );
    }

    fn upgradeable_program_account(program_data_address: Pubkey) -> AccountSharedData {
        let account_len = UpgradeableLoaderState::size_of_program();

        let mut account = Account {
            lamports: Rent::default().minimum_balance(account_len).max(1),
            data: vec![0; account_len],
            owner: bpf_loader_upgradeable::id(),
            executable: true,
            rent_epoch: 0,
        };

        account
            .set_state(&UpgradeableLoaderState::Program {
                programdata_address: program_data_address,
            })
            .unwrap();

        AccountSharedData::from(account)
    }

    fn program_data_account(path: &str, slot: Slot) -> AccountSharedData {
        let program_data = read_file(path);

        let program_data_len =
            UpgradeableLoaderState::size_of_programdata_metadata() + program_data.len();

        let mut program_data_account = Account {
            lamports: Rent::default().minimum_balance(program_data_len).max(1),
            data: vec![0; program_data_len],
            owner: bpf_loader_upgradeable::id(),
            executable: true,
            rent_epoch: 0,
        };

        program_data_account
            .set_state(&UpgradeableLoaderState::ProgramData {
                slot,
                upgrade_authority_address: None,
            })
            .unwrap();

        program_data_account.data[UpgradeableLoaderState::size_of_programdata_metadata()..]
            .copy_from_slice(&program_data);

        AccountSharedData::from(program_data_account)
    }

    async fn simulate_transaction(
        context: &mut ProgramTestContext,
        program_id: Pubkey,
    ) -> solana_banks_interface::BanksTransactionResultWithSimulation {
        let tx = Transaction::new_signed_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &[],
                vec![AccountMeta::new_readonly(program_id, false)],
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.simulate_transaction(tx).await.unwrap()
    }

    async fn simulate_transaction_with_account(
        context: &mut ProgramTestContext,
        program_id: Pubkey,
        account_address: Pubkey,
    ) -> solana_banks_interface::BanksTransactionResultWithSimulation {
        let tx = Transaction::new_signed_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &[],
                vec![
                    AccountMeta::new_readonly(program_id, false),
                    AccountMeta::new_readonly(account_address, false),
                ],
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.simulate_transaction(tx).await.unwrap()
    }
}
