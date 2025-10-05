// SPDX-License-Identifier: MIT
use anchor_lang::{prelude::*, system_program};
// use anchor_spl::token::{self, Mint, MintTo, TokenAccount, Token, Burn};
use anchor_spl::{
    token_2022::{self},
    token_interface::{Burn, Mint, MintTo, Token2022, TokenAccount},
};

use crate::errors::CustomError;

#[account]
pub struct CurveConfiguration {
    pub admin: Pubkey,
    pub protocol_fees: u64,
    pub creator_premint: u64,
    pub creator_fees: u64,
}

impl CurveConfiguration {
    pub const SEED: &'static str = "CurveConfiguration";

    // Discriminator (8) + Pubkey (32) + protocol_fees (8) + creator_premint (8) + creator_fees (8)
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 8 + 8 + 8;

    pub fn new(admin: Pubkey, protocol_fees: u64, creator_premint: u64, creator_fees: u64) -> Self {
        Self {
            admin,
            protocol_fees,
            creator_premint,
            creator_fees,
        }
    }
}

#[account]
pub struct Pool {
    pub creator: Pubkey,
    pub token: Pubkey,
    pub total_supply: u64,
    pub reserve_sol: u64,
    pub bump: u8,
    pub fee_recipients: Vec<FeeRecipient>,
    pub total_fee_share_percentage: u16, // total fee share percentage (0-100)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FeeRecipientType {
    User, // Normal user address
    Pool, // Pool address, used to distribute fees
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FeeRecipient {
    pub recipient_type: FeeRecipientType,
    pub address: Pubkey,
    pub share_percentage: u8, // fee share (0-100)
    pub total_fee: u64,
    pub unclaimed_fee: u64,
}

impl Pool {
    pub const POOL_SEED_PREFIX: &'static str = "liquidity_pool";
    pub const SOL_VAULT_PREFIX: &'static str = "liquidity_sol_vault";
    pub const SOL_FEE_PREFIX: &'static str = "liquidity_sol_fee_vault";

    // Discriminator (8) + creator (32) + token (32) + total_supply (8) + reserve_sol (8) +
    // bump (1) + Vec length (4) + recipients + total_fee_share (2).
    //
    // FeeRecipient serializes as:
    // recipient_type (1 enum tag) + address (32) + share_percentage (1) +
    // total_fee (8) + unclaimed_fee (8) = 50 bytes.
    pub const MAX_FEE_RECIPIENTS: usize = 10;
    pub const ACCOUNT_SIZE: usize =
        8 + 32 + 32 + 8 + 8 + 1 + 4 + (50 * Self::MAX_FEE_RECIPIENTS) + 2;

    pub fn new(creator: Pubkey, token: Pubkey, bump: u8) -> Self {
        Self {
            creator,
            token,
            total_supply: 0_u64,
            reserve_sol: 0_u64,
            bump,
            fee_recipients: Vec::new(),
            total_fee_share_percentage: 0,
        }
    }
}

pub trait PoolAccount<'info> {
    fn buy_with_vault(
        &mut self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _protocol_account: &AccountInfo<'info>,
        _pool_creator: &AccountInfo<'info>,
        _pool_sol_vault: &mut AccountInfo<'info>,
        _pool_sol_fee_vault: &mut AccountInfo<'info>,
        _token_mint: &mut InterfaceAccount<'info, Mint>,
        _recipient_token_account: &mut InterfaceAccount<'info, TokenAccount>,
        _amount: u64,
        _authority: &Signer<'info>,
        _original_pool_sol_vault: &mut AccountInfo<'info>,
        _token_program: &Program<'info, Token2022>,
        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        Ok(())
    }

    fn buy(
        &mut self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _protocol_account: &AccountInfo<'info>,
        _pool_creator: &AccountInfo<'info>,
        _token_accounts: (
            &mut InterfaceAccount<'info, Mint>,         // Token mint
            &mut InterfaceAccount<'info, TokenAccount>, // Pool token account
            &mut InterfaceAccount<'info, TokenAccount>, // User token account
        ),
        _pool_sol_vault: &mut AccountInfo<'info>,
        _pool_sol_fee_vault: &mut AccountInfo<'info>,
        _amount: u64,
        _authority: &Signer<'info>,
        _token_program: &Program<'info, Token2022>,
        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        Ok(())
    }

    fn sell(
        &mut self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _protocol_account: &AccountInfo<'info>,
        _pool_creator: &AccountInfo<'info>,
        _token_accounts: (
            &mut InterfaceAccount<'info, Mint>,         // Token mint
            &mut InterfaceAccount<'info, TokenAccount>, // Pool token account
            &mut InterfaceAccount<'info, TokenAccount>, // User token account
        ),
        _pool_sol_vault: &mut AccountInfo<'info>,
        _pool_sol_fee_vault: &mut AccountInfo<'info>,
        _amount: u64,
        _authority: &Signer<'info>,
        _token_program: &Program<'info, Token2022>,
        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        Ok(())
    }

    // fn transfer_sol_from_pool_to_user(
    //     &self,
    //     _from: &mut AccountInfo<'info>, // pool sol vault
    //     _to: &AccountInfo<'info>,       //  user
    //     _amount: u64,
    // ) -> Result<()> {
    //     Ok(())
    // }

    fn transfer_sol_from_pool(
        &self,
        _from: &mut AccountInfo<'info>,
        _to: &AccountInfo<'info>,
        _amount: u64,
    ) -> Result<()> {
        Ok(())
    }

    fn transfer_sol_to_pool(
        &self,
        _from: &Signer<'info>,
        _to: &AccountInfo<'info>,
        _amount: u64,
        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        Ok(())
    }

    fn transfer_sol_to_user(
        &self,
        _from: &Signer<'info>,
        _to: &AccountInfo<'info>,
        _amount: u64,
        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        Ok(())
    }

    fn distribute_fee_percentage(
        &mut self,
        _fee_share_percentage: u8,
        _recipient_type: FeeRecipientType,
        _recipient: &AccountInfo<'info>,
    ) -> Result<()> {
        Ok(())
    }

    fn clean_zero_fee_recipient(&mut self) -> Result<()> {
        Ok(())
    }

    fn distribute_fees(
        &mut self,
        _is_buy: bool,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _price: u64,
        _authority: &Signer<'info>,
        _protocol_account: &AccountInfo<'info>,
        _pool_creator: &AccountInfo<'info>,
        _pool_sol_vault: &mut AccountInfo<'info>,
        _pool_sol_fee_vault: &mut AccountInfo<'info>,
        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        Ok(())
    }

    fn get_buy_price_after_fees(
        &self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _supply: u64,
        _amount: u64,
    ) -> u64 {
        0
    }

    fn get_sell_price_after_fees(
        &self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _supply: u64,
        _amount: u64,
    ) -> u64 {
        0
    }

    fn get_sell_price(
        &self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _supply: u64,
        _amount: u64,
    ) -> u64 {
        0
    }

    fn get_buy_price(
        &self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _supply: u64,
        _amount: u64,
    ) -> u64 {
        0
    }

    fn get_tokens_for_sol(
        &self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _supply: u64,
        _sol_amount: u64,
    ) -> u64 {
        0
    }

    fn get_tokens_for_sol_after_fees(
        &self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _supply: u64,
        _sol_amount: u64,
    ) -> u64 {
        0
    }

    fn get_price(
        &self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _supply: u64,
        _amount: u64,
    ) -> u64 {
        0
    }

    fn curve(
        &self,
        _curve_configuration: &Account<'info, CurveConfiguration>,
        _amount: u64,
    ) -> u128 {
        0
    }
}

impl<'info> PoolAccount<'info> for Account<'info, Pool> {
    fn buy_with_vault(
        &mut self,
        curve_configuration: &Account<'info, CurveConfiguration>,
        protocol_account: &AccountInfo<'info>,
        pool_creator: &AccountInfo<'info>,
        pool_sol_vault: &mut AccountInfo<'info>,
        pool_sol_fee_vault: &mut AccountInfo<'info>,
        token_mint: &mut InterfaceAccount<'info, Mint>,
        recipient_token_account: &mut InterfaceAccount<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        original_pool_sol_vault: &mut AccountInfo<'info>,
        token_program: &Program<'info, Token2022>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        if amount == 0 {
            return Err(CustomError::InvalidAmount.into());
        }

        let price = self.get_buy_price(curve_configuration, self.total_supply, amount);
        msg!(
            "buy price: {}, total_supply: {}, amount: {}",
            price,
            self.total_supply,
            amount
        );
        if price == 0 {
            return Err(CustomError::InvalidAmount.into());
        }

        // check if the user has enough sol
        if original_pool_sol_vault.lamports() < price {
            return Err(CustomError::InsufficientBalance.into());
        }

        // transfer sol to pool
        msg!(
            "start transfer sol to pool, transfer amount: {}, user balance: {}",
            price,
            original_pool_sol_vault.lamports()
        );
        self.transfer_sol_from_pool(original_pool_sol_vault, pool_sol_vault, price)?;

        // distribute fees
        msg!("start distribute fees");
        self.distribute_fees(
            false, // transfer sol from original pool to user
            curve_configuration,
            price,
            authority,
            protocol_account,
            pool_creator,
            original_pool_sol_vault,
            pool_sol_fee_vault,
            system_program,
        )?;

        // Mint the purchased shares to the claimant. Without this, supply and
        // the Token-2022 mint supply would diverge.
        token_2022::mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                MintTo {
                    mint: token_mint.to_account_info(),
                    to: recipient_token_account.to_account_info(),
                    authority: self.to_account_info(),
                },
                &[&[
                    Pool::POOL_SEED_PREFIX.as_bytes(),
                    self.token.as_ref(),
                    &[self.bump],
                ]],
            ),
            amount,
        )?;

        let price_after_fees =
            self.get_buy_price_after_fees(curve_configuration, self.total_supply, amount);

        self.total_supply += amount;
        self.reserve_sol += price;

        emit!(BuyTokensEvent {
            pool: self.key(),
            user: authority.key(),
            user_token_account: recipient_token_account.key(),
            amount_tokens: amount,
            price: price_after_fees,
        });

        Ok(())
    }

    fn buy(
        &mut self,
        curve_configuration: &Account<'info, CurveConfiguration>,
        protocol_account: &AccountInfo<'info>,
        pool_creator: &AccountInfo<'info>,
        token_accounts: (
            &mut InterfaceAccount<'info, Mint>,         // Token mint
            &mut InterfaceAccount<'info, TokenAccount>, // Pool token account
            &mut InterfaceAccount<'info, TokenAccount>, // User token account
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        pool_sol_fee_vault: &mut AccountInfo<'info>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token2022>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        if amount == 0 {
            return Err(CustomError::InvalidAmount.into());
        }

        let price = self.get_buy_price(curve_configuration, self.total_supply, amount);
        msg!(
            "buy price: {}, total_supply: {}, amount: {}",
            price,
            self.total_supply,
            amount
        );
        if price == 0 {
            return Err(CustomError::InvalidAmount.into());
        }

        // check if the user has enough sol
        if authority.lamports() < price {
            return Err(CustomError::InsufficientBalance.into());
        }

        // transfer sol to pool
        msg!(
            "start transfer sol to pool, transfer amount: {}, user balance: {}",
            price,
            authority.lamports()
        );
        self.transfer_sol_to_pool(authority, pool_sol_vault, price, system_program)?;

        // distribute fees
        msg!("start distribute fees");
        self.distribute_fees(
            true,
            curve_configuration,
            price,
            authority,
            protocol_account,
            pool_creator,
            pool_sol_vault,
            pool_sol_fee_vault,
            system_program,
        )?;

        // Mint tokens to user
        msg!("start minting tokens");
        token_2022::mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                MintTo {
                    mint: token_accounts.0.to_account_info(),
                    to: token_accounts.2.to_account_info(),
                    authority: self.to_account_info(),
                },
                &[&[
                    Pool::POOL_SEED_PREFIX.as_bytes(),
                    token_accounts.0.key().as_ref(),
                    &[self.bump],
                ]],
            ),
            amount,
        )?;
        msg!("Tokens minted successfully");

        let price_after_fees =
            self.get_buy_price_after_fees(curve_configuration, self.total_supply, amount);

        self.total_supply += amount;
        self.reserve_sol += price;

        emit!(BuyTokensEvent {
            pool: self.key(),
            user: authority.key(),
            user_token_account: token_accounts.2.key(),
            amount_tokens: amount,
            price: price_after_fees,
        });

        Ok(())
    }

    fn sell(
        &mut self,
        curve_configuration: &Account<'info, CurveConfiguration>,
        protocol_account: &AccountInfo<'info>,
        pool_creator: &AccountInfo<'info>,
        token_accounts: (
            &mut InterfaceAccount<'info, Mint>,         // Token mint
            &mut InterfaceAccount<'info, TokenAccount>, // Pool token account
            &mut InterfaceAccount<'info, TokenAccount>, // User token account
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        pool_sol_fee_vault: &mut AccountInfo<'info>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token2022>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        if amount == 0 {
            return Err(CustomError::InvalidAmount.into());
        }

        if self.total_supply - curve_configuration.creator_premint < amount {
            msg!(
                "Supply not enough: {}, creator_premint: {}, amount: {}",
                self.total_supply,
                curve_configuration.creator_premint,
                amount
            );
            return Err(CustomError::SupplyNotEnough.into());
        }

        // Burn tokens from user
        msg!("Start burning tokens");
        token_2022::burn(
            CpiContext::new(
                token_program.to_account_info(),
                Burn {
                    mint: token_accounts.0.to_account_info(),
                    from: token_accounts.2.to_account_info(),
                    authority: authority.to_account_info(),
                },
            ),
            amount,
        )?;
        msg!("Tokens burned successfully");

        let price = self.get_sell_price(curve_configuration, self.total_supply, amount);

        if price > self.reserve_sol {
            msg!("price is greater than reserve sol");
            return Err(CustomError::PoolInsufficientFunds.into());
        }

        // calculate total fee = creator fee + protocol fee
        let total_fee = (price as u128
            * (curve_configuration.creator_fees + curve_configuration.protocol_fees) as u128
            / 100) as u64;

        // transfer sol from pool to seller
        msg!(
            "transfer sol from pool to user, transfer amount: {}",
            price - total_fee
        );
        self.transfer_sol_from_pool(pool_sol_vault, authority, price - total_fee)?;

        self.distribute_fees(
            false,
            curve_configuration,
            price,
            authority,
            protocol_account,
            pool_creator,
            pool_sol_vault,
            pool_sol_fee_vault,
            system_program,
        )?;

        let price_after_fees =
            self.get_sell_price_after_fees(curve_configuration, self.total_supply, amount);

        self.reserve_sol -= price;
        self.total_supply -= amount;

        emit!(SellTokensEvent {
            pool: self.key(),
            user: authority.key(),
            user_token_account: token_accounts.2.key(),
            amount_tokens: amount,
            price: price_after_fees,
        });

        Ok(())
    }

    fn distribute_fee_percentage(
        &mut self,
        fee_share_percentage: u8,
        recipient_type: FeeRecipientType,
        recipient: &AccountInfo<'info>,
    ) -> Result<()> {
        if Pool::MAX_FEE_RECIPIENTS <= self.fee_recipients.len() {
            return Err(CustomError::MaxFeeRecipientReached.into());
        }

        let mut total_fee_share_percentage: u16 = 0;
        let mut is_recipient_found = false;
        for fee_recipient in self.fee_recipients.iter_mut() {
            total_fee_share_percentage += fee_recipient.share_percentage as u16;
            if fee_recipient.address == recipient.key() {
                total_fee_share_percentage -= fee_recipient.share_percentage as u16;
                fee_recipient.share_percentage = fee_share_percentage;
                is_recipient_found = true;
            }
        }

        if !is_recipient_found {
            self.fee_recipients.push(FeeRecipient {
                recipient_type: recipient_type.clone(),
                address: recipient.key(),
                share_percentage: fee_share_percentage,
                total_fee: 0,
                unclaimed_fee: 0,
            });
        }

        self.total_fee_share_percentage = total_fee_share_percentage + fee_share_percentage as u16;

        if self.total_fee_share_percentage > 100 {
            return Err(CustomError::InvalidFeeSharePercentage.into());
        }

        emit!(DistributeFeeEvent {
            pool: self.key(),
            fee_share_percentage,
            recipient_type,
            recipient: recipient.key(),
        });

        Ok(())
    }

    fn clean_zero_fee_recipient(&mut self) -> Result<()> {
        // if fee_recipient.share_percentage is 0 and fee_recipient.unclaimed_fee is 0, remove the fee recipient
        let removed_recipients: Vec<Pubkey> = self
            .fee_recipients
            .iter()
            .filter(|r| r.share_percentage == 0 && r.unclaimed_fee == 0)
            .map(|r| r.address)
            .collect();

        self.fee_recipients
            .retain(|recipient| recipient.share_percentage > 0 || recipient.unclaimed_fee > 0);

        // Emit events for each removed recipient
        for recipient in removed_recipients {
            emit!(CleanZeroFeeRecipientEvent {
                pool: self.key(),
                fee_recipient: recipient,
            });
        }

        Ok(())
    }

    fn distribute_fees(
        &mut self,
        is_buy: bool,
        curve_configuration: &Account<'info, CurveConfiguration>,
        price: u64,
        authority: &Signer<'info>,
        protocol_account: &AccountInfo<'info>,
        pool_creator: &AccountInfo<'info>,
        pool_sol_vault: &mut AccountInfo<'info>,
        pool_sol_fee_vault: &mut AccountInfo<'info>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        let protocol_fees = price as u128 * curve_configuration.protocol_fees as u128 / 100;

        // transfer protocol fee to protocol account
        msg!(
            "start transfer sol to protocol account, transfer amount: {}",
            protocol_fees
        );
        if is_buy {
            self.transfer_sol_to_user(
                authority,
                protocol_account,
                protocol_fees as u64,
                system_program,
            )?;
        } else {
            self.transfer_sol_from_pool(pool_sol_vault, protocol_account, protocol_fees as u64)?;
        }

        // calculate total creator fee
        let total_creator_fee = price as u128 * curve_configuration.creator_fees as u128 / 100;

        // transfer fee to pool creator
        let pool_creator_fees =
            total_creator_fee * (100 - self.total_fee_share_percentage as u64) as u128 / 100;
        msg!(
            "start transfer sol to creator account, transfer amount: {}",
            pool_creator_fees
        );
        if is_buy {
            self.transfer_sol_to_user(
                authority,
                pool_creator,
                pool_creator_fees as u64,
                system_program,
            )?;
        } else {
            self.transfer_sol_from_pool(pool_sol_vault, pool_creator, pool_creator_fees as u64)?;
        }

        // transfer remaining fee to pool sol fee vault
        msg!(
            "start transfer sol to pool sol fee vault, transfer amount: {}",
            total_creator_fee - pool_creator_fees
        );
        if is_buy {
            self.transfer_sol_to_pool(
                authority,
                pool_sol_fee_vault,
                (total_creator_fee - pool_creator_fees) as u64,
                system_program,
            )?;
        } else {
            self.transfer_sol_from_pool(
                pool_sol_vault,
                pool_sol_fee_vault,
                (total_creator_fee - pool_creator_fees) as u64,
            )?;
        }

        // update fee recipient config
        for fee_recipient in self.fee_recipients.iter_mut() {
            let fee_share = total_creator_fee * fee_recipient.share_percentage as u128 / 100;
            fee_recipient.total_fee += fee_share as u64;
            fee_recipient.unclaimed_fee += fee_share as u64;
        }

        Ok(())
    }

    // fn transfer_sol_from_pool_to_user(
    //     &self,
    //     from: &mut AccountInfo<'info>, // pool sol vault
    //     to: &AccountInfo<'info>,       //  user
    //     amount: u64,
    // ) -> Result<()> {
    //     msg!(
    //         "transfer sol from pool to user, transfer amount: {}",
    //         amount
    //     );
    //     from.sub_lamports(amount)?;
    //     to.add_lamports(amount)?;
    //     msg!("sol transferred from pool to user");
    //     Ok(())
    // }

    fn transfer_sol_from_pool(
        &self,
        from: &mut AccountInfo<'info>, // pool sol vault
        to: &AccountInfo<'info>,       //  user
        amount: u64,
    ) -> Result<()> {
        from.sub_lamports(amount)?;
        to.add_lamports(amount)?;

        Ok(())
    }

    fn transfer_sol_to_pool(
        &self,
        from: &Signer<'info>,
        to: &AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    fn transfer_sol_to_user(
        &self,
        from: &Signer<'info>,
        to: &AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    fn get_buy_price_after_fees(
        &self,
        curve_configuration: &Account<'info, CurveConfiguration>,
        supply: u64,
        amount: u64,
    ) -> u64 {
        let price = self.get_buy_price(curve_configuration, supply, amount);
        let creator_fees = price * curve_configuration.creator_fees / 100;
        let protocol_fees = price * curve_configuration.protocol_fees / 100;
        price + creator_fees + protocol_fees
    }

    fn get_sell_price_after_fees(
        &self,
        curve_configuration: &Account<'info, CurveConfiguration>,
        supply: u64,
        amount: u64,
    ) -> u64 {
        let price = self.get_sell_price(curve_configuration, supply, amount);
        let creator_fees = price * curve_configuration.creator_fees / 100;
        let protocol_fees = price * curve_configuration.protocol_fees / 100;
        price - creator_fees - protocol_fees
    }

    fn get_tokens_for_sol_after_fees(
        &self,
        curve_configuration: &Account<'info, CurveConfiguration>,
        supply: u64,
        sol_amount: u64,
    ) -> u64 {
        let total_fee_percentage =
            curve_configuration.creator_fees + curve_configuration.protocol_fees;

        if total_fee_percentage == 0 {
            return self.get_tokens_for_sol(curve_configuration, supply, sol_amount);
        }

        let fee_amount = (sol_amount as u128 * total_fee_percentage as u128 / 100) as u64;

        let fee_amount = if fee_amount >= sol_amount {
            sol_amount - 1 // At least 1 lamport for buying
        } else {
            fee_amount
        };

        let actual_sol_amount = sol_amount - fee_amount;

        msg!(
            "sol_amount: {}, fee_amount: {}, actual_sol_amount: {}",
            sol_amount,
            fee_amount,
            actual_sol_amount
        );

        self.get_tokens_for_sol(curve_configuration, supply, actual_sol_amount)
    }

    fn get_sell_price(
        &self,
        curve_configuration: &Account<'info, CurveConfiguration>,
        supply: u64,
        amount: u64,
    ) -> u64 {
        self.get_price(curve_configuration, supply - amount, amount)
    }

    fn get_buy_price(
        &self,
        curve_configuration: &Account<'info, CurveConfiguration>,
        supply: u64,
        amount: u64,
    ) -> u64 {
        self.get_price(curve_configuration, supply, amount)
    }

    fn get_tokens_for_sol(
        &self,
        curve_configuration: &Account<'info, CurveConfiguration>,
        supply: u64,
        sol_amount: u64,
    ) -> u64 {
        msg!(
            "get tokens for sol amount: {}, current supply: {}",
            sol_amount,
            supply
        );

        // convert sol amount to curve unit
        let target_price_delta = (sol_amount as u128) * 1_000_000_000 * 5_000;

        // for curve f(x) = (x - c)², when we require f(supply + amount) - f(supply) = target_price_delta
        // we can expand to: (supply + amount - c)² - (supply - c)² = target_price_delta
        // further expand: (supply - c)² + 2(supply - c)*amount + amount² - (supply - c)² = target_price_delta
        // simplify to: 2(supply - c)*amount + amount² = target_price_delta
        // this is a quadratic equation: amount² + 2(supply - c)*amount - target_price_delta = 0
        // use the quadratic formula: amount = [-2(supply - c) + sqrt(4(supply - c)² + 4*target_price_delta)]/2
        // simplify to: amount = -b + sqrt(b² + target_price_delta), where b = supply - c

        let creator_premint = curve_configuration.creator_premint as u128;
        let effective_supply = if supply > curve_configuration.creator_premint {
            supply as u128 - creator_premint
        } else {
            0u128
        };

        // calculate the coefficient of the quadratic equation
        let b = effective_supply;

        // calculate the discriminant
        let discriminant = (b * b + target_price_delta) as f64;
        if discriminant < 0.0 {
            // it should not happen, but just in case
            return 1;
        }

        // the correct formula: (-b + sqrt(b^2 + target_price_delta))
        // since our b is actually positive, so the result is sqrt(b^2 + target_price_delta) - b
        let amount_f64 = discriminant.sqrt() - b as f64;

        // ensure the result is positive
        if amount_f64 <= 0.0 {
            return 1;
        }

        // floor and convert to u64
        let amount_u64 = amount_f64.floor() as u64;

        // ensure at least 1
        let result = if amount_u64 < 1 { 1 } else { amount_u64 };

        msg!("calculated tokens for sol: {}", result);
        result
    }

    fn get_price(
        &self,
        curve_configuration: &Account<'info, CurveConfiguration>,
        supply: u64,
        amount: u64,
    ) -> u64 {
        msg!("get price with supply: {}, amount: {}", supply, amount);
        let price = (self.curve(curve_configuration, supply + amount)
            - self.curve(curve_configuration, supply))
            / 1_000_000_000
            / 5_000;
        price as u64
    }

    fn curve(&self, curve_configuration: &Account<'info, CurveConfiguration>, amount: u64) -> u128 {
        if amount <= curve_configuration.creator_premint {
            return 0;
        }
        (amount as u128 - curve_configuration.creator_premint as u128)
            * (amount as u128 - curve_configuration.creator_premint as u128)
    }
}

#[event]
pub struct SellTokensEvent {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_token_account: Pubkey,
    pub amount_tokens: u64,
    pub price: u64,
}

#[event]
pub struct BuyTokensEvent {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_token_account: Pubkey,
    pub amount_tokens: u64,
    pub price: u64,
}

#[event]
pub struct DistributeFeeEvent {
    pub pool: Pubkey,
    pub fee_share_percentage: u8,
    pub recipient_type: FeeRecipientType,
    pub recipient: Pubkey,
}

#[event]
pub struct CleanZeroFeeRecipientEvent {
    pub pool: Pubkey,
    pub fee_recipient: Pubkey,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_recipient() -> FeeRecipient {
        FeeRecipient {
            recipient_type: FeeRecipientType::User,
            address: Pubkey::default(),
            share_percentage: 1,
            total_fee: 0,
            unclaimed_fee: 0,
        }
    }

    #[test]
    fn fee_recipient_is_50_bytes() {
        assert_eq!(sample_recipient().try_to_vec().unwrap().len(), 50);
    }

    #[test]
    fn pool_account_size_fits_max_recipients() {
        let mut pool = Pool::new(Pubkey::default(), Pubkey::default(), 0);
        for _ in 0..Pool::MAX_FEE_RECIPIENTS {
            pool.fee_recipients.push(sample_recipient());
        }
        let serialized = pool.try_to_vec().unwrap().len();
        assert!(
            8 + serialized <= Pool::ACCOUNT_SIZE,
            "serialized pool needs {} bytes, allocated {}",
            8 + serialized,
            Pool::ACCOUNT_SIZE
        );
    }
}
