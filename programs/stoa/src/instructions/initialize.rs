// SPDX-License-Identifier: MIT
use anchor_lang::prelude::*;

use crate::{errors::CustomError, state::*};

pub fn initialize(
    ctx: Context<InitializeCurveConfiguration>,
    protocol_fees: u64,
    creator_premint: u64,
    creator_fees: u64,
) -> Result<()> {
    let dex_config = &mut ctx.accounts.curve_configuration_account;

    if protocol_fees > 100 {
        return Err(CustomError::InvalidFee.into());
    }

    if creator_fees > 100 {
        return Err(CustomError::InvalidFee.into());
    }

    // The two fees are deducted together on sells, so their sum must not
    // exceed 100%. Checking each one independently is not enough.
    let total_fees = protocol_fees
        .checked_add(creator_fees)
        .ok_or(CustomError::InvalidFee)?;
    if total_fees > 100 {
        return Err(CustomError::InvalidFee.into());
    }

    dex_config.set_inner(CurveConfiguration::new(
        ctx.accounts.admin.key(),
        protocol_fees,
        creator_premint,
        creator_fees,
    ));
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeCurveConfiguration<'info> {
    #[account(
        init,
        payer = admin,
        space = CurveConfiguration::ACCOUNT_SIZE,
        seeds = [CurveConfiguration::SEED.as_bytes()],
        bump
    )]
    pub curve_configuration_account: Box<Account<'info, CurveConfiguration>>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
