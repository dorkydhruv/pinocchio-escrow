use pinocchio::{
    account_info::AccountInfo,
    no_allocator,
    program_entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use crate::instructions::{ self, ProgramInstruction };

// This is the entrypoint for the program.
program_entrypoint!(process_instruction);
//Do not allocate memory.
no_allocator!();

#[inline(always)]
fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8]
) -> ProgramResult {
    let (ix_disc, data) = data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    match ProgramInstruction::try_from(ix_disc) {
        Ok(ProgramInstruction::MakeInstruction) => {
            instructions::make::process_make_instruction(accounts, data)
        }
        Ok(ProgramInstruction::TakeInstruction) => {
            instructions::take::process_take_instruction(accounts, data)
        }
        Ok(ProgramInstruction::RefundInstruction) => { todo!() }
        Err(_) => { Err(ProgramError::InvalidInstructionData) }
    }
}
