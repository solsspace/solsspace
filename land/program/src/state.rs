use {
    crate::{
        error::LandError,
    },
    arrayref::{array_mut_ref},
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        entrypoint::ProgramResult,
        account_info::AccountInfo,
        program_error::ProgramError,
        borsh::try_from_slice_unchecked,
        pubkey::Pubkey,
        program_pack::{Pack, Sealed},
    },
};

//
// Land Plane Account
//

pub const LAND_PLANE_ACC_DATA_LEN: usize =
1 + // verison
8 + // next_x
8 + // next_y
8;  // depth

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum LandPlaneVersion {
    Uninitialised,
    V1,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct LandPlane {
    pub version: LandPlaneVersion,
    pub next_x: u64,       // 8 bytes
    pub next_z: u64,       // 8 bytes
    pub depth: u64,        // 8 bytes
    // TODO: add an optional owner
    // TODO: add an optional max depth prop
}

impl LandPlane {
    pub fn from_account_info(a: &AccountInfo) -> Result<LandPlane, ProgramError> {
        let data: &[u8] = &a.data.borrow_mut();

        // confirm that given data length is as expected
        if data.len() != LAND_PLANE_ACC_DATA_LEN {
            return Err(LandError::IncorrectDataSize.into());
        }
        
        // otherwise parse
        let result: LandPlane = try_from_slice_unchecked(data)?;

        // and return the result
        Ok(result)
    }

    /// Increment_mint increments the land plane to the
    /// co-ordinate of the next piece of land that will 
    /// be minted.
    /// 
    /// NOTE!!  This function should not be called on an uninitialised
    ///         land plane. i.e. check must be done prior to being called
    ///         in processor.
    /// 
    pub fn increment_mint(&mut self) -> ProgramResult {
        // The first time execution reaches here for some
        // value of self.depth:
        // assert!(true, next_x == self.depth);
        // assert!(true, self.next_z == 0);

        // while next_z is less than depth...
        if self.next_z < self.depth {
            // increment next_z
            self.next_z = self.next_z + 1;

            // Each time exection reaches here:
            // assert!(true, next_x == self.depth);
            // assert!(true, self.next_z < self.depth);

            // Incrementation complete.
            return Ok(())
        }

        // Each time exection reaches here:
        // assert!(true, next_x >= 0);
        // assert!(true, self.next_z == self.depth);

        // while next_x is greater than zero...
        if self.next_x > 0 {
            // decrement next_x
            self.next_x = self.next_x - 1;

            // Each time exection reaches here:
            // assert!(true, next_x > 0);
            // assert!(true, self.next_z == self.depth);

            // Incrementation complete.            
            return Ok(())
        }

        // Execution reaches here ONCE at each depth
        // and it indicates that:
        // assert!(true, next_x == 0);
        // assert!(true, self.next_z == self.depth);

        // Check if land has maxed out
        if self.depth == u64::MAX {
            return Err(LandError::LandComplete.into());
        }

        // Increment depth
        self.depth = self.depth + 1;

        // and reset next_x and next_z
        self.next_x = self.depth;
        self.next_z = 0;

        // done
        Ok(())
    }
}

//
// Land Asset Account
//
pub const LAND_ASSET_ACC_PREFIX: &str = "solsspace-land";

pub const LAND_ASSET_ACC_DATA_LEN: usize =
1 + // verison
32; // mint_pubkey

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum LandAssetVersion {
    Uninitialised,
    V1,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct LandAsset {
    pub version: LandAssetVersion,
    pub mint_pubkey: Pubkey,
}

impl LandAsset {
    pub fn from_account_info(a: &AccountInfo) -> Result<LandAsset, ProgramError> {
        let data: &[u8] = &a.data.borrow_mut();

        // confirm that given data length is as expected
        if data.len() != LAND_ASSET_ACC_DATA_LEN {
            return Err(LandError::IncorrectDataSize.into());
        }
        
        // otherwise parse
        let result: LandAsset = try_from_slice_unchecked(data)?;

        // and return the result
        Ok(result)
    }
}

impl Sealed for LandAsset {}

impl Pack for LandAsset {
    const LEN: usize = LAND_ASSET_ACC_DATA_LEN;
    fn unpack_from_slice(data: &[u8]) -> Result<Self, ProgramError> {
        // confirm that given data length is as expected
        if data.len() != LAND_ASSET_ACC_DATA_LEN {
            return Err(LandError::IncorrectDataSize.into());
        }

        // otherwise parse
        let result: LandAsset = try_from_slice_unchecked(data)?;

        // and return the result
        Ok(result)
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, LAND_ASSET_ACC_DATA_LEN];
        let res = self.try_to_vec().unwrap();
        for (i, x) in res.iter().enumerate() {
            dst[i] = *x
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_land_plane_increment_land() {
        for (no_of_increments, expected_lp) in vec![
            (
                8,
                LandPlane{
                    version: LandPlaneVersion::V1,
                    next_x: 0,
                    next_z: 2,
                    depth: 2,
                },
            ),
            (
                11,
                LandPlane{
                    version: LandPlaneVersion::V1,
                    next_x: 3,
                    next_z: 2,
                    depth: 3,
                },
            ),
            ] {

            // initialse new land plane
            let mut lp = LandPlane{
                version: LandPlaneVersion::V1,
                next_x: 0,
                next_z: 0,
                depth: 0,
            };

            // increment given number of times
            for _i in 0..no_of_increments {
                assert_eq!(Ok(()), lp.increment_mint());            
            };

            // confirm result as expected
            assert_eq!(expected_lp, lp);       
        }
    }
}