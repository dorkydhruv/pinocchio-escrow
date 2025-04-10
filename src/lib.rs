#![no_std]
#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
pub mod instructions;
pub mod state;
mod utils;
pinocchio_pubkey::declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");
