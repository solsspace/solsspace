use {
    crate::{
        error::LandError,
        instruction::{
            LandInstruction,
        },
        state::{
            LAND_PLANE_ACC_DATA_LEN,
            LAND_ASSET_ACC_PREFIX,
            LandPlane,
            LandPlaneVersion,
            LandAsset,
            LandAssetVersion,            
        },
    },
    borsh::{BorshDeserialize,BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        sysvar::{rent::Rent, Sysvar},
        pubkey::Pubkey,
    },
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = LandInstruction::try_from_slice(input)?;
    match instruction {
        LandInstruction::InitialiseLandPlane => {
            msg!("Instruction: Initialise Land Plane");
            process_initialise_land_plane(
                accounts,
            )
        },
        LandInstruction::InitialiseLandAsset => {
            msg!("Instruction: Initialise Land Asset");
            process_initialise_land_asset(
                accounts,
            )
        }
        LandInstruction::MintNext => {
            msg!("Instruction: Mint Next");
            process_mint_next(
                program_id,
                accounts,
            )
        }
    }
}

/// Initialise a new Land Plane
pub fn process_initialise_land_plane(
    accounts: &[AccountInfo],
) -> ProgramResult {
    // prepare an account info iterator and get a handle
    // on required accounts
    let account_info_iter = &mut accounts.iter();
    let land_plane_acc_info = next_account_info(account_info_iter)?;
    let rent_acc_info = next_account_info(account_info_iter)?;

    // parse the uninitialised land plane account state
    let mut land_plane_acc_state = LandPlane::from_account_info(land_plane_acc_info)?;

    // confirm that account is not already in use
    if land_plane_acc_state.version != LandPlaneVersion::Uninitialised {
        return Err(LandError::AlreadyInUse.into());
    }

    // parse rent from rent account info
    let rent = &Rent::from_account_info(rent_acc_info)?;    

    // confirm that given land plane account is rent exempt
    if !rent.is_exempt(land_plane_acc_info.lamports(), LAND_PLANE_ACC_DATA_LEN) {
        return Err(LandError::NotRentExempt.into());
    }    

    // initialise values
    land_plane_acc_state.version = LandPlaneVersion::V1;
    land_plane_acc_state.next_x = 0;
    land_plane_acc_state.next_z = 0;
    land_plane_acc_state.depth = 0;

    // then serialize the land plane account state again
    land_plane_acc_state.serialize(&mut *land_plane_acc_info.data.borrow_mut())?;
    
    Ok(())
}

/// Initialise a new Land Asset
pub fn process_initialise_land_asset(
    accounts: &[AccountInfo],
) -> ProgramResult {
    Ok(())
}

/// Mint next piece of land
pub fn process_mint_next(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    // prepare an account info iterator and get a handle
    // on required accounts
    let account_info_iter = &mut accounts.iter();
    let nft_assoc_token_acc_owner_acc_info = next_account_info(account_info_iter)?;
    let land_asset_acc_info = next_account_info(account_info_iter)?;
    let land_plane_acc_info = next_account_info(account_info_iter)?;
    let _nft_assoc_token_acc_owner_acc_info = next_account_info(account_info_iter)?;
    let _nft_mint_acc_info = next_account_info(account_info_iter)?;

    // confirm that given nft associated token acc owner is a signatory
    // on the transaction
    if !nft_assoc_token_acc_owner_acc_info.is_signer {
        return Err(LandError::SignatureError.into());
    }

    // parse land plane account state and confirm
    // that the given account has been initialised
    let land_plane_acc_state = LandPlane::from_account_info(land_plane_acc_info)?;
    if land_plane_acc_state.version == LandPlaneVersion::Uninitialised {
        return Err(LandError::LandPlaneAccUninitialised.into());
    }

    // derive expected PDA for next piece of land
    let (next_land_asset_acc_key, _) = Pubkey::find_program_address(
        &[
            LAND_ASSET_ACC_PREFIX.as_bytes(),
            land_plane_acc_info.key.as_ref(),
            &land_plane_acc_state.next_x.to_le_bytes(),
            &land_plane_acc_state.next_z.to_le_bytes(),
        ],
        program_id,
    );

    // confirm correct land_asset_acc was provided
    if land_asset_acc_info.key != &next_land_asset_acc_key {
        return Err(LandError::InvalidLandAssetAccKey.into());
    }

    // parse land asset account state and confirm
    // that the given account has been initialised
    let land_asset_acc_state = LandAsset::from_account_info(land_asset_acc_info)?;
    if land_asset_acc_state.version == LandAssetVersion::Uninitialised {
        return Err(LandError::LandAssetAccUninitialised.into());
    }    

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate :: {
        instruction::{
            initialize_land_plane,
            mint_next,
        },
        state::{
            LAND_ASSET_ACC_DATA_LEN,
        },
    };
    use solana_program::{
        system_program,
        program_error::{PrintProgramError, ProgramError},
        instruction::Instruction,
    };
    use solana_sdk::account::{
        create_account_for_test, create_is_signer_account_infos, Account as SolanaAccount,
    };

    ///
    /// testing utils
    /// 

    fn return_land_error_as_program_error() -> ProgramError {
        LandError::IncorrectDataSize.into()
    }

    fn rent_sysvar() -> SolanaAccount {
        create_account_for_test(&Rent::default())
    }

    fn do_process_instruction(
        instruction: Instruction,
        accounts: Vec<&mut SolanaAccount>,
    ) -> ProgramResult {
        let mut meta = instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();

        let account_infos = create_is_signer_account_infos(&mut meta);
        process_instruction(&instruction.program_id, &account_infos, &instruction.data)
    }  
    
    fn land_plane_minimum_balance() -> u64 {
        Rent::default().minimum_balance(LAND_PLANE_ACC_DATA_LEN)
    }    

    ///
    /// tests
    /// 

    #[test]
    fn test_print_error() {
        let error = return_land_error_as_program_error();
        error.print::<LandError>();
    }    

    #[test]
    fn test_initialise_land_plane_account() {
        let program_id = crate::id();
        let land_plane_acc_key = Pubkey::new_unique();
        let mut land_plane_acc = SolanaAccount::new(42, LAND_PLANE_ACC_DATA_LEN, &program_id);
        let mut rent_sysvar = rent_sysvar();

        //
        // given account to be initialised is not rent exempt 
        //
        assert_eq!(
            Err(LandError::NotRentExempt.into()),
            do_process_instruction(
                initialize_land_plane(&program_id, &land_plane_acc_key).unwrap(),
                vec![&mut land_plane_acc, &mut rent_sysvar]
            )
        );
        // correct rent
        land_plane_acc.lamports = land_plane_minimum_balance();

        // instruction completes successfully
        do_process_instruction(
            initialize_land_plane(&program_id, &land_plane_acc_key).unwrap(),
            vec![&mut land_plane_acc, &mut rent_sysvar]
        )
        .unwrap();

        //
        // trying to call initialise again fails
        //
        assert_eq!(
            Err(LandError::AlreadyInUse.into()),
            do_process_instruction(
                initialize_land_plane(&program_id, &land_plane_acc_key).unwrap(),
                vec![&mut land_plane_acc, &mut rent_sysvar]
            )
        );        
    }

    #[test]
    fn test_mint_next() {
        let program_id = crate::id();

        let nft_assoc_token_acc_owner_acc_pubkey = Pubkey::new_unique();
        let mut nft_assoc_token_acc_owner_acc = SolanaAccount::new(1, 0, &system_program::id());        

        let land_asset_acc_wrong_pubkey = Pubkey::new_unique();
        let mut land_asset_acc = SolanaAccount::new(1, LAND_ASSET_ACC_DATA_LEN, &system_program::id());

        let land_plane_acc_pubkey = Pubkey::new_unique();
        let mut land_plane_acc = SolanaAccount::new(42, LAND_PLANE_ACC_DATA_LEN, &program_id);

        let nft_assoc_token_acc_pubkey = Pubkey::new_unique();
        let mut nft_assoc_token_acc = SolanaAccount::new(1, 0, &system_program::id());                

        let nft_mint_acc_pubkey = Pubkey::new_unique();
        let mut nft_mint_acc = SolanaAccount::new(1, 0, &system_program::id());

        //
        // land plane account not initialised
        //
        assert_eq!(
            Err(LandError::LandPlaneAccUninitialised.into()),
            do_process_instruction(
                mint_next(
                    &program_id,
                    &nft_assoc_token_acc_owner_acc_pubkey,
                    &land_asset_acc_wrong_pubkey,
                    &land_plane_acc_pubkey,
                    &nft_assoc_token_acc_pubkey,
                    &nft_mint_acc_pubkey,
                ).unwrap(),
                vec![
                    &mut nft_assoc_token_acc_owner_acc,
                    &mut land_asset_acc,
                    &mut land_plane_acc,
                    &mut nft_assoc_token_acc,
                    &mut nft_mint_acc,
                    ]
            )
        );
        
        // initialise land plane account
        let land_plane = LandPlane{
            version: LandPlaneVersion::V1,
            next_x: 100,
            next_z: 21,
            depth: 100,
        };
        land_plane_acc.data = land_plane.try_to_vec().unwrap();

        //
        // invalid land asset acc key
        //
        assert_eq!(
            Err(LandError::InvalidLandAssetAccKey.into()),
            do_process_instruction(
                mint_next(
                    &program_id,
                    &nft_assoc_token_acc_owner_acc_pubkey,
                    &land_asset_acc_wrong_pubkey,
                    &land_plane_acc_pubkey,
                    &nft_assoc_token_acc_pubkey,
                    &nft_mint_acc_pubkey,
                ).unwrap(),
                vec![
                    &mut nft_assoc_token_acc_owner_acc,
                    &mut land_asset_acc,
                    &mut land_plane_acc,
                    &mut nft_assoc_token_acc,
                    &mut nft_mint_acc,
                    ]
            )
        );

        // generate correct land asset account for next piece of land
        let (land_asset_acc_pubkey, _) = Pubkey::find_program_address(
            &[
                LAND_ASSET_ACC_PREFIX.as_bytes(),
                land_plane_acc_pubkey.as_ref(),
                &land_plane.next_x.to_le_bytes(),
                &land_plane.next_z.to_le_bytes(),
            ],
            &program_id,
        );

        //
        // land asset account not initialised
        //
        assert_eq!(
            Err(LandError::LandAssetAccUninitialised.into()),
            do_process_instruction(
                mint_next(
                    &program_id,
                    &nft_assoc_token_acc_owner_acc_pubkey,
                    &land_asset_acc_pubkey,
                    &land_plane_acc_pubkey,
                    &nft_assoc_token_acc_pubkey,
                    &nft_mint_acc_pubkey,
                ).unwrap(),
                vec![
                    &mut nft_assoc_token_acc_owner_acc,
                    &mut land_asset_acc,
                    &mut land_plane_acc,
                    &mut nft_assoc_token_acc,
                    &mut nft_mint_acc,
                    ]
            )
        );
    }
}