use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{ self, Pubkey },
    ProgramResult,
};

use crate::{ instructions::MakeEscrow, utils::{ load_acc_mut_unchecked, DataLen } };

#[repr(C)] //keeps the struct layout the same across different architectures
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Escrow {
    pub seed: u8,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub maker: Pubkey,
    pub receive_amount: u64,
    pub bump: u8,
}

impl DataLen for Escrow {
    const LEN: usize = Self::LEN;
}

impl Escrow {
    pub const SEED: &'static str = "escrow";
    pub const LEN: usize = 1 + 32 + 32 + 32 + 8 + 1;

    pub fn validate_pda(bump: u8, pda: &Pubkey, owner: &Pubkey) -> Result<(), ProgramError> {
        let seeds = &[Self::SEED.as_bytes(), owner, &[bump]];
        let derived = pubkey::create_program_address(seeds, &crate::ID)?;
        if derived != *pda {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(())
    }

    pub fn intialize(escrow_acc: &AccountInfo, ix_data: &MakeEscrow) -> ProgramResult {
        let escrow_state = (unsafe {
            load_acc_mut_unchecked::<Escrow>(escrow_acc.borrow_mut_data_unchecked())
        })?;
        escrow_state.seed = ix_data.seed;
        escrow_state.mint_a = ix_data.mint_a;
        escrow_state.mint_b = ix_data.mint_b;
        escrow_state.maker = *escrow_acc.key();
        escrow_state.receive_amount = ix_data.recieve_amount;
        escrow_state.bump = ix_data.bump;
        Ok(())
    }

    pub fn from_account_info(account_info: &AccountInfo) -> &mut Self {
        assert_eq!(account_info.data_len(), Self::LEN);
        assert_eq!(unsafe { account_info.owner() }, &crate::ID);
        unsafe { &mut *(account_info.borrow_mut_data_unchecked().as_ptr() as *mut Self) }
    }

    pub fn from_account_info_readable(account_info: &AccountInfo) -> &Self {
        unsafe {
            assert_eq!(account_info.data_len(), Escrow::LEN);
            assert_eq!(account_info.owner(), &crate::ID);
            &*(account_info.borrow_data_unchecked().as_ptr() as *const Self)
        }
    }
}
