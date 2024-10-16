//! Sandbox program.

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction, system_program,
};

solana_program::declare_id!("SandboxProgram11111111111111111111111111111");

solana_program::entrypoint!(process_instruction);

pub fn program_derived_address() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"seed"], &crate::id())
}

pub fn instruction(
    from_address: &Pubkey,
    to_address: &Pubkey,
    discriminator: u8,
    amount: u64,
) -> Instruction {
    if discriminator != 0 && discriminator != 1 {
        panic!("Invalid discriminator");
    }
    let accounts = vec![
        AccountMeta::new(*from_address, false),
        AccountMeta::new(*to_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let mut data = vec![discriminator];
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let mut accounts_iter = &mut accounts.iter();

    let from_info = next_account_info(&mut accounts_iter)?;
    let to_info = next_account_info(&mut accounts_iter)?;

    let (pda, bump) = program_derived_address();

    if from_info.key != &pda {
        msg!("Invalid from address");
        return Err(ProgramError::InvalidArgument);
    }

    match input.split_first() {
        Some((&0, rest)) if rest.len() == 8 => {
            msg!("Instruction: Transfer with CPI to System");

            let amount = u64::from_le_bytes(rest.try_into().unwrap());

            invoke_signed(
                &system_instruction::transfer(from_info.key, to_info.key, amount),
                &[from_info.clone(), to_info.clone()],
                &[&[&b"seed"[..], &[bump]]],
            )
        }
        Some((&1, rest)) if rest.len() == 8 => {
            msg!("Instruction: Transfer with direct lamport manipulation");

            let amount = u64::from_le_bytes(rest.try_into().unwrap());

            let new_from_lamports = from_info.lamports() - amount;
            let new_to_lamports = to_info.lamports() + amount;

            **from_info.try_borrow_mut_lamports()? = new_from_lamports;
            **to_info.try_borrow_mut_lamports()? = new_to_lamports;

            Ok(())
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
