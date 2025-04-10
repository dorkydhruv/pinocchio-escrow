use pinocchio::{
    account_info::AccountInfo,
    instruction::{ Seed, Signer },
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_token::{
    instructions::{ CloseAccount, TransferChecked },
    ID as TOKEN_ID,
};

use crate::{ state::Escrow, utils::load_acc_unchecked };

pub fn process_refund_instructions(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [
        maker_acc,
        mint_a_acc,
        maker_mint_a_ata,
        escrow_acc,
        escrow_vault,
        _rent_sysvar,
        _token_program,
        _system_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Check signers
    if !maker_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let escrow_state = (unsafe {
        load_acc_unchecked::<Escrow>(escrow_acc.borrow_data_unchecked())
    })?;

    //Check authority
    unsafe {
        assert_eq!(*mint_a_acc.owner(), TOKEN_ID);
        assert_eq!(*escrow_vault.owner(), *escrow_acc.key());
        assert_eq!(escrow_acc.owner(), &crate::id());
    }

    // Check maker account
    if *maker_acc.key() != escrow_state.maker {
        return Err(ProgramError::IncorrectAuthority);
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
    // Transfer the tokens back to the maker
    (TransferChecked {
        amount: escrow_state.receive_amount,
        authority: escrow_acc,
        decimals: 6,
        from: escrow_vault,
        mint: mint_a_acc,
        to: maker_mint_a_ata,
    }).invoke_signed(&[signers.clone()])?;

    // Close the escrow vault account
    (CloseAccount {
        account: escrow_vault,
        destination: maker_acc,
        authority: escrow_acc,
    }).invoke_signed(&[signers.clone()])?;

    // Close the escrow account
    unsafe {
        *maker_acc.borrow_mut_lamports_unchecked() += *escrow_acc.borrow_lamports_unchecked();
        escrow_acc.close()?;
    }
    Ok(())
}
