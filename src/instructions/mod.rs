use pinocchio::program_error::ProgramError;
pub mod make;
pub mod take;
pub mod refund;
pub use make::*;
pub use take::*;
pub use refund::*;
#[repr(u8)]
pub enum ProgramInstruction {
    MakeInstruction,
    TakeInstruction,
    RefundInstruction,
}

impl TryFrom<&u8> for ProgramInstruction {
    type Error = ProgramError;
    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ProgramInstruction::MakeInstruction),
            1 => Ok(ProgramInstruction::TakeInstruction),
            2 => Ok(ProgramInstruction::RefundInstruction),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
