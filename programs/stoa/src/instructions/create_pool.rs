// SPDX-License-Identifier: MIT
use anchor_lang::prelude::*;
use anchor_lang::system_program;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{
        mint_to, set_authority, spl_token_2022::instruction::AuthorityType, MintTo, SetAuthority,
    },
    token_interface::{Mint, Token2022, TokenAccount},
};

use crate::state::*;

pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // create pool sol vault if not exists
    if ctx.accounts.pool_sol_vault.data_is_empty() {
        msg!("pool_sol_vault is empty, creating pool_sol_vault");
        let vault_bump = ctx.bumps.pool_sol_vault;
        let token_mint_key = ctx.accounts.token_mint.key();
        let seeds = &[
            Pool::SOL_VAULT_PREFIX.as_bytes(),
            token_mint_key.as_ref(),
            &[vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(0);

        system_program::create_account(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::CreateAccount {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.pool_sol_vault.to_account_info(),
                },
                signer_seeds,
            ),
            lamports,
            0,
            ctx.program_id,
        )?;
    }

    // create pool sol fee vault if not exists
    if ctx.accounts.pool_sol_fee_vault.data_is_empty() {
        msg!("pool_sol_fee_vault is empty, creating pool_sol_fee_vault");
        let fee_vault_bump = ctx.bumps.pool_sol_fee_vault;
        let token_mint_key = ctx.accounts.token_mint.key();
        let seeds = &[
            Pool::SOL_FEE_PREFIX.as_bytes(),
            token_mint_key.as_ref(),
            &[fee_vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(0);

        system_program::create_account(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::CreateAccount {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.pool_sol_fee_vault.to_account_info(),
                },
                signer_seeds,
            ),
            lamports,
            0,
            ctx.program_id,
        )?;
    }

    // transfer mint authority to pool
    msg!("transferring mint authority to pool");
    set_authority(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.payer.to_account_info(),
                account_or_mint: ctx.accounts.token_mint.to_account_info(),
            },
        ),
        AuthorityType::MintTokens,
        Some(pool.key()),
    )?;

    // premint creator tokens
    let creator_premint = ctx.accounts.curve_configuration.creator_premint;
    msg!("start minting tokens");
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: pool.to_account_info(),
            },
            &[&[
                Pool::POOL_SEED_PREFIX.as_bytes(),
                ctx.accounts.token_mint.key().as_ref(),
                &[ctx.bumps.pool],
            ]],
        ),
        creator_premint,
    )?;

    // set pool info
    pool.set_inner(Pool::new(
        ctx.accounts.payer.key(),
        ctx.accounts.token_mint.key(),
        ctx.bumps.pool,
    ));

    pool.total_supply = creator_premint;

    emit!(PoolCreated {
        creator: ctx.accounts.payer.key(),
        token_mint: ctx.accounts.token_mint.key(),
        pool: ctx.accounts.pool.key(),
        pool_sol_vault: ctx.accounts.pool_sol_vault.key(),
        pool_sol_fee_vault: ctx.accounts.pool_sol_fee_vault.key(),
        pool_token_account: ctx.accounts.pool_token_account.key(),
    });

    Ok(())
}

#[event]
pub struct PoolCreated {
    pub creator: Pubkey,
    pub token_mint: Pubkey,
    pub pool: Pubkey,
    pub pool_sol_vault: Pubkey,
    pub pool_sol_fee_vault: Pubkey,
    pub pool_token_account: Pubkey,
}

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(
        mut,
        seeds = [CurveConfiguration::SEED.as_bytes()],
        bump
   )]
    pub curve_configuration: Box<Account<'info, CurveConfiguration>>,

    #[account(
        init,
        payer = payer,
        space = Pool::ACCOUNT_SIZE,
        seeds = [Pool::POOL_SEED_PREFIX.as_bytes(), token_mint.key().as_ref()],
        bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
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
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
