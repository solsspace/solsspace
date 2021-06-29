use crate::{check_program_account};
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        program_error::{ProgramError},
        pubkey::Pubkey,
        instruction::{AccountMeta, Instruction},
        sysvar,
    },
};

/// Instructions supported by the Metadata program.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum LandInstruction {
    /// Initialise Land Plane Account
    /// 
    /// The `InitialiseLandPlane` instruction requires no signers and MUST be
    /// included within the same Transaction as the system program's `CreateAccount`
    /// instruction that creates the account being initialized. Otherwise another
    /// party can initialise the account.
    /// 
    /// Accounts expected by this instruction:
    /// 
    /// 0. `[writable] land_place_acc`
    ///     Land plane account to initialise.
    /// 1. `[] rent_sysvar_acc`
    InitialiseLandPlane,


    /// Initialise Land Asset Account
    /// 
    /// Initialise land asset account before using it in a mint next
    /// instruction.
    /// 
    /// Accounts expected by this instruction:
    /// 
    /// 1. `[signer] rent_payer_acc`
    ///     Key of account responsible for paying required rent for the new
    ///     land_asset_acc
    /// 2. `[writable] land_asset_acc`
    ///     Key of new land asset account.
    ///     This key should be a PDA of:
    ///     (['solsspace-land', land_plane_acc_pubkey, x, y], land_program_acc_pubkey)
    ///     Typically this would correspond to the next piece of land that will be minted.
    /// 3. `[] system_program_acc`
    /// 4. `[] rent_sysvar_acc`
    InitialiseLandAsset,

    /// Mint Land Pience
    /// 
    /// The `MintNext` instruction will mint the next piece of land
    /// linking it to the given SPL NFT. This renders the owner of
    /// the NFT the owner of the new piece of land.
    /// 
    /// Accounts expected by this instruction:
    ///
    /// 0. `[signer] nft_assoc_token_acc_owner_acc`
    ///     A normal system account that is the owner of the SPL NFT holding associate token
    ///     account. A signature is required for this account to confirm that the given owner
    ///     would like to associate the new piece of land with their NFT.
    /// 1. `[writable] land_asset_acc`
    ///     This account should already exist and have been initialised through invocation
    ///     of the InitialiseLandAsset method on the land program.
    ///     This account should be a PDA corresponding to the next piece of land.
    ///     i.e. PDA of (['solsspace-land', land_plane_acc_pubkey, x, y], land_program_acc_pubkey)
    /// 2. `[writable] land_plane_acc`
    ///     Public key of the land plane account from which the next piece of land will be minted.
    /// 3. `[] nft_assoc_token_acc`
    ///     Public key of an SPL NFT holding account. Should be owned by given
    ///     `nft_assoc_token_acc_owner` and should hold a balance of 1.
    /// 4. `[] nft_mint_acc`
    ///     The SPL NFT Mint account.
    MintNext,
}

/// Creates an `InitialiseLandPlane` instruction.
pub fn initialize_land_plane(
    land_program_acc_pubkey: &Pubkey,
    land_plane_acc_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    check_program_account(land_program_acc_pubkey)?;
    let data = LandInstruction::InitialiseLandPlane.try_to_vec().unwrap();

    // prepare list of account to pass to the instruction
    let accounts = vec![
        // 1st
        // Addresses requiring signatures are 1st, and in the following order:
        //
        // those that require write access
        // those that require read-only access

        // 2nd
        // Addresses not requiring signatures are 2nd, and in the following order:
        //
        // those that require write access        
        AccountMeta::new(*land_plane_acc_pubkey, false),
        // those that require read-only access
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction {
        program_id: *land_program_acc_pubkey,
        accounts,
        data,
    })
}

/// Creates a `MintNext` instruction.
/// 
/// * `land_program_acc_pubkey`
///     Public key of the land program account - aka. program ID.
/// * `[signer] nft_assoc_token_acc_owner_pubkey`
///     Public key of the normal system account that is the owner of the given NFT holding SPL
///     associate token account. A signature is required for this account to confirm
///     that the given owner would like to associate the new piece of land with their NFT.
/// * `[writable] land_asset_acc_pubkey`
///     This key should be a PDA corresponding to the next piece of land.
///     i.e. PDA of (['solsspace-land', land_plane_acc_pubkey, x, y], land_program_acc_pubkey)
/// * `[writable] land_plane_acc_pubkey`
///     Public key of the land plane account from which the next piece of land will be minted.
/// * `[] nft_assoc_token_acc_pubkey`
///     Public key of an SPL NFT holding account. Should be owned by given
///     `nft_assoc_token_acc_owner_pubkey` and should hold a balance of 1.
/// * `[] nft_mint_acc_pubkey`
///     Public key of the SPL NFT Mint account.
pub fn mint_next(
    land_program_acc_pubkey: &Pubkey,
    nft_assoc_token_acc_owner_pubkey: &Pubkey,
    land_asset_acc_pubkey: &Pubkey,
    land_plane_acc_pubkey: &Pubkey,
    nft_assoc_token_acc_pubkey: &Pubkey,
    nft_mint_acc_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    // confirm given program id is correct
    check_program_account(land_program_acc_pubkey)?;

    // prepare data to pass in instruction
    let data = LandInstruction::MintNext.try_to_vec().unwrap();

    // prepare list of accounts to pass in instruction
    let accounts = vec![
        // 1st
        // Addresses requiring signatures are 1st, and in the following order:
        //
        // those that require write access
        // those that require read-only access
        AccountMeta::new_readonly(*nft_assoc_token_acc_owner_pubkey, true),
        
        // 2nd
        // Addresses not requiring signatures are 2nd, and in the following order:
        //
        // those that require write access
        AccountMeta::new(*land_asset_acc_pubkey, false),
        AccountMeta::new(*land_plane_acc_pubkey, false),
        // those that require read-only access
        AccountMeta::new_readonly(*nft_assoc_token_acc_pubkey, false),
        AccountMeta::new_readonly(*nft_mint_acc_pubkey, false),
    ];

    // return instruction
    Ok(Instruction {
        program_id: *land_program_acc_pubkey,
        accounts,
        data,
    })
}