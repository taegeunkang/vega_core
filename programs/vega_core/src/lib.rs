use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("GRM6VqaFBXMGZiXvUMvRYerfNKskU4Emz8ZsEhQYCgAG");

mod instructions;
mod states;
mod utils;
use crate::instructions::*;

#[program]
pub mod vega_core {

    use super::*;

    /// Initialize a Config account for setting fee rate.
    pub fn initialize(ctx: Context<Initialize>, fee_rate: u8) -> ProgramResult {
        return instructions::initialize::handler(ctx, fee_rate);
    }

    /// Initialize Pool account that contain token.
    /// Pool account has fee rate, mint, vault (ata) , lp supply, bump
    pub fn create_pool(ctx: Context<CreateStakingPool>) -> ProgramResult {
        return instructions::create_staking_pool::handler(ctx);
    }

    /// Initialize UserPoolinfo account.
    /// amount - amount of token that signer want to stake
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> ProgramResult {
        return instructions::deposit::handler(ctx, amount);
    }

    /// withdraw token that signer deposited.
    /// reward caculated by after - before / 2 sec * amount of token per 2sec
    pub fn withdraw(ctx: Context<Withdraw>) -> ProgramResult {
        return instructions::withdraw::handler(ctx);
    }

    /// buy token.
    /// rate : 1 token == 0.1 SOL
    pub fn buy(ctx: Context<Buy>, amount: u64) -> ProgramResult {
        return instructions::buy::handler(ctx, amount);
    }

    /// sell token
    pub fn sell(ctx: Context<Sell>, amount: u64) -> ProgramResult {
        return instructions::sell::handler(ctx, amount);
    }

    /// trading SOL/LP
    /// available long , short both
    /// if user get profit, user can additional reward when withdraw.
    pub fn trade_in(ctx: Context<TradeIn>, way: u8, amount: u64) -> ProgramResult {
        return instructions::trade_in::handler(ctx, way, amount);
    }

    /// close positon
    pub fn trade_out(ctx: Context<TradeOut>) -> ProgramResult {
        return instructions::trade_out::handler(ctx);
    }
}
