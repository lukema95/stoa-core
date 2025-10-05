// SPDX-License-Identifier: MIT
use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, Token2022, TokenAccount},
};

use crate::errors::CustomError;
use crate::state::*;

pub fn buy_tokens(ctx: Context<BuyTokens>, amount_tokens: u64) -> Result<()> {
    if ctx.accounts.protocol_account.key() != ctx.accounts.curve_configuration_account.admin {
        return Err(CustomError::InvalidProtocolAccount.into());
    }

    let pool = &mut ctx.accounts.pool;

    let token_accounts = (
        &mut *ctx.accounts.token_mint,
        &mut *ctx.accounts.pool_token_account,
        &mut *ctx.accounts.user_token_account,
    );

    pool.buy(
        &ctx.accounts.curve_configuration_account,
        &ctx.accounts.protocol_account,
        &ctx.accounts.pool_creator,
        token_accounts,
        &mut ctx.accounts.pool_sol_vault,
        &mut ctx.accounts.pool_sol_fee_vault,
        amount_tokens,
        &ctx.accounts.user,
        &ctx.accounts.token_program,
        &ctx.accounts.system_program,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(
      mut,
      seeds = [CurveConfiguration::SEED.as_bytes()],
      bump
    )]
    pub curve_configuration_account: Box<Account<'info, CurveConfiguration>>,

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
      associated_token::mint = token_mint,
      associated_token::authority = pool,
      associated_token::token_program = token_program,
    )]
    pub pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK:
    #[account(
      mut,
      seeds = [Pool::SOL_VAULT_PREFIX.as_bytes(), token_mint.key().as_ref()],
      bump
    )]
    pub pool_sol_vault: AccountInfo<'info>,

    /// CHECK:
    #[account(
      mut,
      seeds = [Pool::SOL_FEE_PREFIX.as_bytes(), token_mint.key().as_ref()],
      bump
    )]
    pub pool_sol_fee_vault: AccountInfo<'info>,

    #[account(
      mut,
      associated_token::mint = token_mint,
      associated_token::authority = user,
      associated_token::token_program = token_program,
    )]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK:
    #[account(mut)]
    pub protocol_account: AccountInfo<'info>,

    /// CHECK: Must match `pool.creator`; validated by the address constraint.
    #[account(mut, address = pool.creator)]
    pub pool_creator: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
