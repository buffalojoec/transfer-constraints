// #![cfg(feature = "test-sbf")]

use {
    mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk},
    solana_sdk::{
        account::AccountSharedData, instruction::InstructionError, program_error::ProgramError,
        pubkey::Pubkey, system_instruction, system_program,
    },
};

#[test]
fn system_transfers() {
    let mollusk = Mollusk::default();

    let starting_balance = 100_000_000;
    let transfer_amount = 50_000_000;

    let from = sandbox_program::program_derived_address().0;
    let to = Pubkey::new_unique();

    // SUCCESS CASE: System account with no data.
    mollusk.process_and_validate_instruction(
        &system_instruction::transfer(&from, &to, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 0, &system_program::id()),
            ),
            (to, AccountSharedData::default()),
        ],
        &[
            Check::success(),
            Check::account(&from)
                .lamports(starting_balance - transfer_amount)
                .build(),
            Check::account(&to).lamports(transfer_amount).build(),
        ],
    );

    // FAILURE CASE: System account _with_ data.
    mollusk.process_and_validate_instruction(
        &system_instruction::transfer(&from, &to, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 500, &system_program::id()),
            ),
            (to, AccountSharedData::default()),
        ],
        &[
            // Transfer: `from` must not carry data
            Check::err(ProgramError::InvalidArgument),
        ],
    );

    // FAILURE CASE: Third-party account with no data.
    mollusk.process_and_validate_instruction(
        &system_instruction::transfer(&from, &to, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 0, &sandbox_program::id()),
            ),
            (to, AccountSharedData::default()),
        ],
        &[
            // instruction spent from the balance of an account it does not own
            Check::instruction_err(InstructionError::ExternalAccountLamportSpend),
        ],
    );

    // FAILURE CASE: Third-party account _with_ data.
    mollusk.process_and_validate_instruction(
        &system_instruction::transfer(&from, &to, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 500, &sandbox_program::id()),
            ),
            (to, AccountSharedData::default()),
        ],
        &[
            // Transfer: `from` must not carry data
            Check::err(ProgramError::InvalidArgument),
        ],
    );
}

// CPI to the System program will obviously impose the same constraints as
// invoking it in the top-level.
#[test]
fn program_transfers_via_cpi_to_system() {
    let mollusk = Mollusk::new(&sandbox_program::id(), "sandbox_program");

    let starting_balance = 100_000_000;
    let transfer_amount = 50_000_000;

    let from = sandbox_program::program_derived_address().0;
    let to = Pubkey::new_unique();

    // SUCCESS CASE: System account with no data.
    mollusk.process_and_validate_instruction(
        &sandbox_program::instruction(&from, &to, 0, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 0, &system_program::id()),
            ),
            (to, AccountSharedData::default()),
            keyed_account_for_system_program(),
        ],
        &[
            Check::success(),
            Check::account(&from)
                .lamports(starting_balance - transfer_amount)
                .build(),
            Check::account(&to).lamports(transfer_amount).build(),
        ],
    );

    // FAILURE CASE: System account _with_ data.
    mollusk.process_and_validate_instruction(
        &sandbox_program::instruction(&from, &to, 0, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 500, &system_program::id()),
            ),
            (to, AccountSharedData::default()),
            keyed_account_for_system_program(),
        ],
        &[
            // Transfer: `from` must not carry data
            Check::err(ProgramError::InvalidArgument),
        ],
    );

    // FAILURE CASE: Third-party account with no data.
    mollusk.process_and_validate_instruction(
        &sandbox_program::instruction(&from, &to, 0, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 0, &sandbox_program::id()),
            ),
            (to, AccountSharedData::default()),
            keyed_account_for_system_program(),
        ],
        &[
            // instruction spent from the balance of an account it does not own
            Check::instruction_err(InstructionError::ExternalAccountLamportSpend),
        ],
    );

    // FAILURE CASE: Third-party account _with_ data.
    mollusk.process_and_validate_instruction(
        &sandbox_program::instruction(&from, &to, 0, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 500, &sandbox_program::id()),
            ),
            (to, AccountSharedData::default()),
            keyed_account_for_system_program(),
        ],
        &[
            // Transfer: `from` must not carry data
            Check::err(ProgramError::InvalidArgument),
        ],
    );
}

// However, directly manipulating lamports is a different story.
// Notice that the owner of the destination doesn't matter. Programs can credit
// any account, they just can't debit.
#[test]
fn program_transfers_via_direct_lamport_manipulation() {
    let mollusk = Mollusk::new(&sandbox_program::id(), "sandbox_program");

    let starting_balance = 100_000_000;
    let transfer_amount = 50_000_000;

    let from = sandbox_program::program_derived_address().0;
    let to = Pubkey::new_unique();

    // SUCCESS CASE: Program-owned account with no data.
    mollusk.process_and_validate_instruction(
        &sandbox_program::instruction(&from, &to, 1, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 0, &sandbox_program::id()),
            ),
            (to, AccountSharedData::default()),
            keyed_account_for_system_program(),
        ],
        &[
            Check::success(),
            Check::account(&from)
                .lamports(starting_balance - transfer_amount)
                .build(),
            Check::account(&to).lamports(transfer_amount).build(),
        ],
    );

    // SUCCESS CASE: Program-owned account _with_ data.
    mollusk.process_and_validate_instruction(
        &sandbox_program::instruction(&from, &to, 1, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 500, &sandbox_program::id()),
            ),
            (to, AccountSharedData::default()),
            keyed_account_for_system_program(),
        ],
        &[
            Check::success(),
            Check::account(&from)
                .lamports(starting_balance - transfer_amount)
                .build(),
            Check::account(&to).lamports(transfer_amount).build(),
        ],
    );

    // FAILURE CASE: System account with no data.
    mollusk.process_and_validate_instruction(
        &sandbox_program::instruction(&from, &to, 1, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 0, &system_program::id()),
            ),
            (to, AccountSharedData::default()),
            keyed_account_for_system_program(),
        ],
        &[
            // instruction spent from the balance of an account it does not own
            Check::instruction_err(InstructionError::ExternalAccountLamportSpend),
        ],
    );

    // FAILURE CASE: System account _with_ data.
    mollusk.process_and_validate_instruction(
        &sandbox_program::instruction(&from, &to, 1, transfer_amount),
        &[
            (
                from,
                AccountSharedData::new(starting_balance, 500, &system_program::id()),
            ),
            (to, AccountSharedData::default()),
            keyed_account_for_system_program(),
        ],
        &[
            // instruction spent from the balance of an account it does not own
            Check::instruction_err(InstructionError::ExternalAccountLamportSpend),
        ],
    );
}
