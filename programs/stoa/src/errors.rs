// SPDX-License-Identifier: MIT
use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Invalid fee")]
    InvalidFee,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Insufficient funds")]
    InsufficientFunds,

    #[msg("Invalid owner")]
    InvalidOwner,

    #[msg("Invalid protocol account")]
    InvalidProtocolAccount,

    #[msg("Supply not enough")]
    SupplyNotEnough,

    #[msg("Pool insufficient funds")]
    PoolInsufficientFunds,

    #[msg("Invalid recipient")]
    InvalidRecipient,

    #[msg("Invalid fee share percentage")]
    InvalidFeeSharePercentage,

    #[msg("Max fee recipient reached")]
    MaxFeeRecipientReached,
}
