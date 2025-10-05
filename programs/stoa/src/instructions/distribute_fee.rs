// SPDX-License-Identifier: MIT
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::CustomError;
use crate::state::*;

pub fn distribute_user_fee(
    ctx: Context<DistributeUserFee>,
    fee_share_percentage: u8,
) -> Result<()> {
    // only the pool creator can distribute the fee
    if ctx.accounts.pool.creator != ctx.accounts.user.key() {
        return Err(CustomError::InvalidOwner.into());
    }

    if fee_share_percentage > 100 {
        return Err(CustomError::InvalidFeeSharePercentage.into());
    }

    let pool = &mut ctx.accounts.pool;
    let recipient = &ctx.accounts.recipient;

    pool.distribute_fee_percentage(fee_share_percentage, FeeRecipientType::User, recipient)?;

    Ok(())
}

pub fn distribute_pool_fee(
    ctx: Context<DistributePoolFee>,
    fee_share_percentage: u8,
) -> Result<()> {
    // only the pool creator can distribute the fee
    if ctx.accounts.pool.creator != ctx.accounts.user.key() {
        return Err(CustomError::InvalidOwner.into());
    }

    if fee_share_percentage > 100 {
        return Err(CustomError::InvalidFeeSharePercentage.into());
    }

    // Check if the target pool is the same as the current pool (prevent self-distribution)
    if ctx.accounts.pool.key() == ctx.accounts.distribute_fee_pool.key() {
        return Err(CustomError::InvalidRecipient.into());
    }

    let pool = &mut ctx.accounts.pool;
    let distribute_fee_pool = &mut ctx.accounts.distribute_fee_pool;

    pool.distribute_fee_percentage(
        fee_share_percentage,
        FeeRecipientType::Pool,
        &distribute_fee_pool.to_account_info(),
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct DistributeUserFee<'info> {
    #[account(
      mut,
      seeds = [Pool::POOL_SEED_PREFIX.as_bytes(), token_mint.key().as_ref()],
      bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK:
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributePoolFee<'info> {
    #[account(
      mut,
      seeds = [Pool::POOL_SEED_PREFIX.as_bytes(), token_mint.key().as_ref()],
      bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
      mut,
      seeds = [Pool::POOL_SEED_PREFIX.as_bytes(), distribute_fee_pool_token_mint.key().as_ref()],
      bump = distribute_fee_pool.bump
    )]
    pub distribute_fee_pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub distribute_fee_pool_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
