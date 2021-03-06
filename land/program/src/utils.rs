use {
    // crate::{
    //     // error::LandError,
    //     // // processor::process_create_metadata_accounts,
    //     // state::{
    //     //     LandPlaneVersion,
    //     //     // get_reservation_list, Data, Edition, Key, MasterEdition, Metadata, EDITION,
    //     //     // MAX_CREATOR_LIMIT, MAX_EDITION_LEN, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
    //     //     // PREFIX,
    //     // },
    // },
    // borsh::{
    //     // BorshDeserialize,
    //     // BorshSerialize
    // },
    solana_program::{
        account_info::AccountInfo,
        // borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        // program_error::ProgramError,
        // program_pack::{IsInitialized, Pack},
        pubkey::Pubkey,
        system_instruction,
        sysvar::{rent::Rent, Sysvar},
    },
    std::convert::TryInto,
};

/// Create account almost from scratch, lifted from
/// https://github.com/solana-labs/solana-program-library/tree/master/associated-token-account/program/src/processor.rs#L51-L98
#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(&payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        &[new_account_info.clone(), system_program_info.clone()],
        &[&signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        &[new_account_info.clone(), system_program_info.clone()],
        &[&signer_seeds],
    )?;

    Ok(())
}

