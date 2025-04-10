use pinocchio::{
    account_info::AccountInfo,
    instruction::{ Seed, Signer },
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_token::{
    instructions::{ CloseAccount, InitializeAccount, TransferChecked },
    ID as TOKEN_ID,
};

use crate::{ state::Escrow, utils::load_acc_unchecked};

pub fn process_take_instruction(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [
        taker_acc,
        maker_acc,
        mint_a_acc,
        mint_b_acc,
        taker_mint_a_ata,
        taker_mint_b_ata,
        maker_mint_b_ata,
        escrow_acc,
        escrow_vault,
        rent_sysvar,
        _token_program,
        _system_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    // Check signers
    if !taker_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let escrow_state = (unsafe {
        load_acc_unchecked::<Escrow>(escrow_acc.borrow_data_unchecked())
    })?;

    // Check maker account
    if *maker_acc.key() != escrow_state.maker {
        return Err(ProgramError::IncorrectAuthority);
    }

    unsafe {
        assert_eq!(mint_a_acc.owner(), &TOKEN_ID);
        assert_eq!(mint_b_acc.owner(), &TOKEN_ID);
    }

    let seed_le_bytes = escrow_state.seed.to_le_bytes();
    let signer_bump = [escrow_state.bump];
    let signer_seeds = [
        Seed::from(Escrow::SEED.as_bytes()),
        Seed::from(maker_acc.key().as_ref()),
        Seed::from(seed_le_bytes.as_ref()),
        Seed::from(&signer_bump[..]),
    ];
    let signers = Signer::from(&signer_seeds[..]);
    // Initialize maker_mint_b_ata and taker_mint_a_ata (if not initialized)
    if maker_mint_b_ata.data_is_empty() {
        (InitializeAccount {
            account: maker_mint_b_ata,
            mint: mint_b_acc,
            owner: maker_acc,
            rent_sysvar,
        }).invoke_signed(&[signers.clone()])?;
    }
    if taker_mint_a_ata.data_is_empty() {
        (InitializeAccount {
            account: taker_mint_a_ata,
            mint: mint_a_acc,
            owner: taker_acc,
            rent_sysvar,
        }).invoke_signed(&[signers.clone()])?;
    }
    // Transfer token from vault to taker_mint_a_ata
    (TransferChecked {
        amount: escrow_state.receive_amount,
        authority: escrow_acc,
        from: escrow_vault,
        mint: mint_a_acc,
        to: taker_mint_a_ata,
        decimals: 6,
    }).invoke_signed(&[signers.clone()])?;
    // Transfer token from taker_mint_b_ata to maker_mint_b_ata
    (TransferChecked {
        amount: escrow_state.receive_amount,
        authority: taker_acc,
        from: taker_mint_b_ata,
        mint: mint_b_acc,
        to: maker_mint_b_ata,
        decimals: 6,
    }).invoke_signed(&[signers.clone()])?;

    // Close the escrow account
    // Close escrow vault account
    (CloseAccount {
        account: escrow_vault,
        destination: maker_acc,
        authority: escrow_acc,
    }).invoke_signed(&[signers.clone()])?;
    // Move the lamports
    unsafe {
        *maker_acc.borrow_mut_lamports_unchecked() += *escrow_acc.borrow_lamports_unchecked();
        escrow_acc.close()?;
    }
    Ok(())
}
