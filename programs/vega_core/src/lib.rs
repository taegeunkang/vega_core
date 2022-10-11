use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("Cbvd322Xp7Qfkf2VvgyCH4cKkJXwypDYsGK8upAm3ZXK");

mod instructions;
mod states;
mod utils;
use crate::instructions::*;

#[program]
pub mod vega_core {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee_rate: u8) -> ProgramResult {
        return instructions::initialize::handler(ctx, fee_rate);
    }

    pub fn buy(ctx: Context<Buy>, amount: u64) -> ProgramResult {
        return instructions::buy::handler(ctx, amount);
    }

    pub fn sell(ctx: Context<Sell>, amount: u64) -> ProgramResult {
        return instructions::sell::handler(ctx, amount);
    }

    pub fn create_pool(ctx: Context<CreateStakingPool>) -> ProgramResult {
        return instructions::create_staking_pool::handler(ctx);
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> ProgramResult {
        return instructions::deposit::handler(ctx, amount);
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> ProgramResult {
        return instructions::withdraw::handler(ctx);
    }

    pub fn trade_in(ctx: Context<TradeIn>, way: u8, amount: u64) -> ProgramResult {
        return instructions::trade_in::handler(ctx, way, amount);
    }

    pub fn trade_out(ctx: Context<TradeOut>) -> ProgramResult {
        return instructions::trade_out::handler(ctx);
    }
}
