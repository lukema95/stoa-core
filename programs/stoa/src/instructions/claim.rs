// SPDX-License-Identifier: MIT
use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, Token2022, TokenAccount},
};

use crate::errors::CustomError;
use crate::state::*;

pub fn claim_user_fees(ctx: Context<ClaimUserFees>) -> Result<()> {
    // Only the fee recipient can claim their own fees
    if ctx.accounts.fee_recipient.key() != ctx.accounts.user.key() {
        return Err(CustomError::InvalidOwner.into());
    }

    let pool = &mut ctx.accounts.pool;
    let recipient = &mut ctx.accounts.fee_recipient;
    let pool_sol_fee_vault = &mut ctx.accounts.pool_sol_fee_vault;
    let mut claim_amount = 0;

    // Calculate the user's fee and transfer it to the user
    for fee_recipient in pool.fee_recipients.iter_mut() {
        if fee_recipient.address == recipient.key() {
            if fee_recipient.recipient_type == FeeRecipientType::User
                && fee_recipient.unclaimed_fee > 0
            {
                let unclaimed_fee = fee_recipient.unclaimed_fee;
                claim_amount = unclaimed_fee;
                fee_recipient.unclaimed_fee = 0;
                // check the pool sol fee vault has enough SOL to pay for the fee
                let pool_sol_fee_vault_balance = pool_sol_fee_vault.lamports();
                if pool_sol_fee_vault_balance < unclaimed_fee {
                    return Err(CustomError::InsufficientFunds.into());
                }
                pool.transfer_sol_from_pool(pool_sol_fee_vault, recipient, unclaimed_fee)?;
            }
            break;
        }
    }

    // If the fee recipient's share_percentage is 0 and unclaimed_fee is 0, remove the fee recipient
    pool.clean_zero_fee_recipient()?;

    emit!(ClaimFeesEvent {
        pool: pool.key(),
        fee_recipient_type: FeeRecipientType::User,
        fee_recipient: recipient.key(),
        sender: ctx.accounts.user.key(),
        claim_amount,
    });

    Ok(())
}

pub fn claim_pool_fees(ctx: Context<ClaimPoolFees>) -> Result<()> {
    // Only the target pool creator can claim pool fees
    if ctx.accounts.distribution_pool.creator != ctx.accounts.user.key() {
        return Err(CustomError::InvalidOwner.into());
    }

    // The protocol fee recipient must be the configured admin.
    if ctx.accounts.protocol_account.key() != ctx.accounts.curve_configuration_account.admin {
        return Err(CustomError::InvalidProtocolAccount.into());
    }

    let pool = &mut ctx.accounts.pool;
    let distribution_pool = &mut ctx.accounts.distribution_pool;

    // Get the recipient index
    let recipient_index = pool
        .fee_recipients
        .iter()
        .position(|recipient| {
            recipient.address == distribution_pool.key()
                && recipient.recipient_type == FeeRecipientType::Pool
                && recipient.unclaimed_fee > 0
        })
        .ok_or(CustomError::InvalidRecipient)?;

    // Get the fee recipient and calculate the token amount
    let fee_recipient = &pool.fee_recipients[recipient_index];
    let token_amount = distribution_pool.get_tokens_for_sol_after_fees(
        &ctx.accounts.curve_configuration_account,
        distribution_pool.total_supply,
        fee_recipient.unclaimed_fee,
    );

    let claim_amount = fee_recipient.unclaimed_fee;

    // Calculate the pool's fee and buy tokens from the pool
    if let Some(fee_recipient) = pool.fee_recipients.get_mut(recipient_index) {
        fee_recipient.unclaimed_fee = 0;

        // Buy tokens from the pool
        msg!("Start to buy pool tokens with vault");
        distribution_pool.buy_with_vault(
            &ctx.accounts.curve_configuration_account,
            &ctx.accounts.protocol_account,
            &ctx.accounts.distribution_pool_creator,
            &mut ctx.accounts.distribution_pool_sol_vault,
            &mut ctx.accounts.distribution_pool_sol_fee_vault,
            &mut ctx.accounts.distribution_pool_token_mint,
            &mut ctx.accounts.distribution_pool_user_token_account,
            token_amount,
            &ctx.accounts.user,
            &mut ctx.accounts.pool_sol_fee_vault, // source fee vault pays for the shares
            &ctx.accounts.token_program,
            &ctx.accounts.system_program,
        )?;
    }

    // If the fee recipient's share_percentage is 0 and unclaimed_fee is 0, remove the fee recipient
    pool.clean_zero_fee_recipient()?;

    emit!(ClaimFeesEvent {
        pool: pool.key(),
        fee_recipient_type: FeeRecipientType::Pool,
        fee_recipient: distribution_pool.key(),
        sender: ctx.accounts.user.key(),
        claim_amount,
    });

    Ok(())
}

#[event]
pub struct ClaimFeesEvent {
    pub pool: Pubkey,
    pub fee_recipient_type: FeeRecipientType,
    pub fee_recipient: Pubkey,
    pub sender: Pubkey,
    pub claim_amount: u64,
}

#[derive(Accounts)]
pub struct ClaimUserFees<'info> {
    #[account(
      mut,
      seeds = [Pool::POOL_SEED_PREFIX.as_bytes(), token_mint.key().as_ref()],
      bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK:
    #[account(
      mut,
      seeds = [Pool::SOL_FEE_PREFIX.as_bytes(), token_mint.key().as_ref()],
      bump
    )]
    pub pool_sol_fee_vault: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub fee_recipient: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct ClaimPoolFees<'info> {
    /*************************************************************
     * The params for the pool that the user is claiming fees from *
     *************************************************************/
    // The pool that the user is claiming fees from
    #[account(
      mut,
      seeds = [Pool::POOL_SEED_PREFIX.as_bytes(), token_mint.key().as_ref()],
      bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    // The pool vault that holds the SOL fees
    /// CHECK:
    #[account(
      mut,
      seeds = [Pool::SOL_FEE_PREFIX.as_bytes(), token_mint.key().as_ref()],
      bump
    )]
    pub pool_sol_fee_vault: AccountInfo<'info>,

    // The token mint for the pool
    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    /*************************************************************
     * The params for the pool that the fees are distributed to  *
     *************************************************************/
    // The curve configuration account for the pool
    #[account(
      mut,
      seeds = [CurveConfiguration::SEED.as_bytes()],
      bump
    )]
    pub curve_configuration_account: Box<Account<'info, CurveConfiguration>>,

    // The pool that the fees are distributed to
    #[account(
      mut,
      seeds = [Pool::POOL_SEED_PREFIX.as_bytes(), distribution_pool_token_mint.key().as_ref()],
      bump = distribution_pool.bump
    )]
    pub distribution_pool: Box<Account<'info, Pool>>,

    // The token mint for the pool
    #[account(mut)]
    pub distribution_pool_token_mint: Box<InterfaceAccount<'info, Mint>>,

    // The pool token account for the pool
    #[account(
      mut,
      associated_token::mint = distribution_pool_token_mint,
      associated_token::authority = distribution_pool,
      associated_token::token_program = token_program,
    )]
    pub distribution_pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // The pool SOL vault for the pool
    /// CHECK:
    #[account(
      mut,
      seeds = [Pool::SOL_VAULT_PREFIX.as_bytes(), distribution_pool_token_mint.key().as_ref()],
      bump
    )]
    pub distribution_pool_sol_vault: AccountInfo<'info>,

    // The pool SOL fee vault for the pool
    /// CHECK:
    #[account(
      mut,
      seeds = [Pool::SOL_FEE_PREFIX.as_bytes(), distribution_pool_token_mint.key().as_ref()],
      bump
    )]
    pub distribution_pool_sol_fee_vault: AccountInfo<'info>,

    // The user's token account for the pool
    #[account(
      mut,
      associated_token::mint = distribution_pool_token_mint,
      associated_token::authority = user,
      associated_token::token_program = token_program,
    )]
    pub distribution_pool_user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // The protocol account for the pool
    /// CHECK:
    #[account(mut)]
    pub protocol_account: AccountInfo<'info>,

    // The pool creator for the distribution pool
    /// CHECK: Must match `distribution_pool.creator`.
    #[account(mut, address = distribution_pool.creator)]
    pub distribution_pool_creator: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
