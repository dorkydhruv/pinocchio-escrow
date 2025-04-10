use pinocchio::{
    account_info::AccountInfo,
    instruction::{ Seed, Signer },
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{ rent::Rent, Sysvar },
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{
    ID as TOKEN_ID,
    instructions::{ InitializeAccount, TransferChecked },
    state::Mint,
};

use crate::{ state::Escrow, utils::{ load_acc_unchecked, load_ix_data, DataLen } };

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MakeEscrow {
    pub seed: u8,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub recieve_amount: u64,
    pub data: [u8; 32],
    pub bump: u8,
}

impl DataLen for Mint {
    const LEN: usize = core::mem::size_of::<Self>();
}

impl DataLen for MakeEscrow {
    const LEN: usize = core::mem::size_of::<Self>();
}

pub fn process_make_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [
        maker_acc,
        mint_a_acc,
        mint_b_acc,
        maker_mint_a_ata,
        escrow_acc,
        escrow_vault,
        rent_sysvar,
        _token_program,
        _system_program,
        _rest @ ..,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    // Check that account is a signer
    if !maker_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // Check that the escrow account is not already initialized
    if !escrow_acc.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let rent = Rent::get()?;
    let ix_data = load_ix_data::<MakeEscrow>(data)?;
    let seed_le_bytes = ix_data.seed.to_le_bytes();
    let signer_bump = [ix_data.bump];
    let signer_seeds = [
        Seed::from(Escrow::SEED.as_bytes()),
        Seed::from(maker_acc.key().as_ref()),
        Seed::from(seed_le_bytes.as_ref()),
        Seed::from(&signer_bump[..]),
    ];
    let signers = Signer::from(&signer_seeds[..]);
    (CreateAccount {
        from: maker_acc,
        to: escrow_acc,
        owner: &crate::id(),
        space: Escrow::LEN as u64,
        lamports: rent.minimum_balance(Escrow::LEN),
    }).invoke_signed(&[signers])?;
    Escrow::intialize(escrow_acc, &ix_data)?;

    // State updates done, now we can create the associated token account for the escrow ie escrow vault
    unsafe {
        assert_eq!(mint_a_acc.owner(), &TOKEN_ID);
        assert_eq!(mint_b_acc.owner(), &TOKEN_ID);
    }

    let mint_a_state = (unsafe { load_acc_unchecked::<Mint>(mint_a_acc.borrow_data_unchecked()) })?;

    let signer = Signer::from(&signer_seeds[..]);

    // Create the associated token account for the escrow vault
    if escrow_vault.data_is_empty() {
        // Only create the account if it is empty that is not intialised
        (InitializeAccount {
            account: escrow_vault,
            mint: mint_a_acc,
            owner: escrow_acc,
            rent_sysvar,
        }).invoke_signed(&[signer.clone()])?;
    }

    // Check the owner is escrow_acc
    unsafe {
        assert_eq!(escrow_vault.owner(), escrow_acc.key());
    }

    // Transfer the tokens from the maker to the escrow vault
    (TransferChecked {
        amount: ix_data.recieve_amount,
        authority: maker_acc,
        decimals: mint_a_state.decimals(), // Should probably derive this too
        from: maker_mint_a_ata,
        mint: mint_a_acc,
        to: escrow_vault,
    }).invoke_signed(&[signer.clone()])?;

    Ok(())
}
