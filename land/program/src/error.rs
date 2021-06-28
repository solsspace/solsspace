//! Error types

use {
    num_derive::FromPrimitive,
    solana_program::{
        decode_error::DecodeError,
        msg,
        program_error::{PrintProgramError, ProgramError},
    },
    thiserror::Error,
};

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum LandError {
    /// Account is already in use
    #[error("Account already in use")]
    AlreadyInUse,

    /// Signature error
    #[error("Signature error")]
    SignatureError,    

    /// Land plane account is uninitialised
    #[error("Land plane account uninitialsed")]
    LandPlaneAccUninitialised,

    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,

    /// IncorrectDataSize 
    #[error("Incorrect data size")]
    IncorrectDataSize,

    /// LandComplete
    #[error("Land Complete")]
    LandComplete,

    /// InvalidLandAssetAccKey
    #[error("Invalid land asset acc key")]
    InvalidLandAssetAccKey,    

    /// Land asset account is uninitialised
    #[error("Land asset account uninitialsed")]
    LandAssetAccUninitialised,    
}

impl PrintProgramError for LandError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<LandError> for ProgramError {
    fn from(e: LandError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for LandError {
    fn type_of() -> &'static str {
        "Land Error"
    }
}