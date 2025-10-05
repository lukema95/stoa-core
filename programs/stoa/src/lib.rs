// SPDX-License-Identifier: MIT
#![allow(unexpected_cfgs)]
#![allow(clippy::too_many_arguments)]
use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use crate::instructions::*;

declare_id!("7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM");

#[program]
pub mod stoa {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializeCurveConfiguration>,
        protocol_fees: u64,
        creator_premint: u64,
        creator_fees: u64,
    ) -> Result<()> {
        instructions::initialize(ctx, protocol_fees, creator_premint, creator_fees)
    }

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        instructions::create_pool(ctx)
    }

    pub fn distribute_user_fee(
        ctx: Context<DistributeUserFee>,
        fee_share_percentage: u8,
    ) -> Result<()> {
        instructions::distribute_user_fee(ctx, fee_share_percentage)
    }

    pub fn distribute_pool_fee(
        ctx: Context<DistributePoolFee>,
        fee_share_percentage: u8,
    ) -> Result<()> {
        instructions::distribute_pool_fee(ctx, fee_share_percentage)
    }

    pub fn claim_user_fees(ctx: Context<ClaimUserFees>) -> Result<()> {
        instructions::claim_user_fees(ctx)
    }

    pub fn claim_pool_fees(ctx: Context<ClaimPoolFees>) -> Result<()> {
        instructions::claim_pool_fees(ctx)
    }

    pub fn buy(ctx: Context<BuyTokens>, amount_tokens: u64) -> Result<()> {
        instructions::buy_tokens(ctx, amount_tokens)
    }

    pub fn sell(ctx: Context<SellTokens>, amount_tokens: u64) -> Result<()> {
        instructions::sell_tokens(ctx, amount_tokens)
    }
}
