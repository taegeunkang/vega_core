use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::system_program;
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint};
use anchor_spl::token::{Token, TokenAccount, Transfer};
use chainlink_solana as chainlink;
declare_id!("Cbvd322Xp7Qfkf2VvgyCH4cKkJXwypDYsGK8upAm3ZXK");

mod states_1;
mod utils;
mod instructions;
mod states;
use states_1::*;
use crate::instructions::*;
use crate::utils::*;

#[program]
pub mod vega_core {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee_rate : u8) -> ProgramResult {
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
        let reward: u64 = ctx.accounts.calc_reward();

        let seeds = &[
            b"pool".as_ref(),
            ctx.accounts.config.to_account_info().key.as_ref(),
            ctx.accounts.mint.key.as_ref(),
            &[ctx.accounts.pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_vault.to_account_info(),
                to: ctx.accounts.user_ata.clone(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_context, reward)?;

        ProgramResult::Ok(())
    }

    pub fn trade_in(ctx: Context<TradeIn>, _way: u8, _amount: u64) -> ProgramResult {
        let _round = chainlink::latest_round_data(
            ctx.accounts.chainlink_program.to_account_info(),
            ctx.accounts.chainlink_feed.to_account_info(),
        )?;

        let _decimals = chainlink::decimals(
            ctx.accounts.chainlink_program.to_account_info(),
            ctx.accounts.chainlink_feed.to_account_info(),
        )?;
        let _current_price = _round.answer as u64;
        msg!(
            "current price : {}, decimals : {}",
            _current_price,
            _decimals
        );
        msg!(
            "trade_in way : {}, current_price : {}, decimals : {}, amount : {}",
            _way,
            _current_price,
            _decimals,
            _amount
        );

        ctx.accounts.init(_way, _amount, _current_price, _decimals);

        ProgramResult::Ok(())
    }

    pub fn trade_out (ctx: Context<TradeOut>) -> ProgramResult {
        let _round = chainlink::latest_round_data(
            ctx.accounts.chainlink_program.to_account_info(),
            ctx.accounts.chainlink_feed.to_account_info(),
        )?;

        let _decimals = chainlink::decimals(
            ctx.accounts.chainlink_program.to_account_info(),
            ctx.accounts.chainlink_feed.to_account_info(),
        )?;
        let _current_price = _round.answer as u64;
        
        let (percentage, pl )= calc_pl(ctx.accounts.trade_info.entry_price, _current_price, ctx.accounts.trade_info.way);
        let _lp_amount = calc_trade_out_lp_amount(ctx.accounts.user_pool_info.current_lp_amount, percentage, pl);
        msg!("user trade out");
        msg!("trade in : {}, trade out : {}", ctx.accounts.user_pool_info.current_lp_amount, _lp_amount);
        ctx.accounts.user_pool_info.current_lp_amount = _lp_amount;

        ProgramResult::Ok(())
    }
}





#[derive(Accounts)]
#[instruction()]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, seeds = [b"pool", config.key().as_ref(), mint.key().as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, seeds=[pool.key().as_ref(), mint.key().as_ref()], bump, constraint = pool_vault.owner == pool.key())]
    pub pool_vault: Box<Account<'info, TokenAccount>>,
    /// CHECK : this is safe
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    /// CHECK : this is safe
    #[account(mut)]
    pub user_ata: AccountInfo<'info>,
    #[account(mut, seeds=[b"user_pool_info", signer.key().as_ref(), pool.key().as_ref()], bump, close = signer)]
    pub user_pool_info: Box<Account<'info, UserPoolInfo>>,
    #[account(mut, seeds=[b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(address = solana_program::sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
    #[account(address = solana_program::sysvar::clock::ID)]
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_way : u8, _amount : u64)]
pub struct TradeIn<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK : this is safe
    #[account(mut)]
    pub pool: AccountInfo<'info>,
    #[account(init, seeds=[b"trade", signer.key().as_ref()], bump, payer = signer, space= 8 + std::mem::size_of::<TradeInfo>())]
    pub trade_info: Box<Account<'info, TradeInfo>>,
    #[account(mut, seeds=[b"user_pool_info", signer.key().as_ref(), pool.key().as_ref()], bump)]
    pub user_pool_info: Box<Account<'info, UserPoolInfo>>,
    /// CHECK: We're reading data from this specified chainlink feed
    pub chainlink_feed: AccountInfo<'info>,
    /// CHECK: This is the Chainlink program library on Devnet
    pub chainlink_program: AccountInfo<'info>,
    #[account(address = solana_program::sysvar::clock::ID)]
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct TradeOut<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK : this is safe
    #[account(mut)]
    pub pool: AccountInfo<'info>,
    #[account(mut, seeds=[b"trade", signer.key().as_ref()], bump, close = signer)]
    pub trade_info: Box<Account<'info, TradeInfo>>,
    #[account(mut, seeds=[b"user_pool_info", signer.key().as_ref(), pool.key().as_ref()], bump)]
    pub user_pool_info: Box<Account<'info, UserPoolInfo>>,
    /// CHECK: We're reading data from this specified chainlink feed
    pub chainlink_feed: AccountInfo<'info>,
    /// CHECK: This is the Chainlink program library on Devnet
    pub chainlink_program: AccountInfo<'info>,
    #[account(address = solana_program::sysvar::clock::ID)]
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}



impl<'info> Withdraw<'info> {
    pub fn calc_reward(&self) -> u64 {
        let reward: u64 = calc_reward_percent(
            self.user_pool_info.deposited_amount,
            self.user_pool_info.deposited_time,
            self.clock.unix_timestamp as u64,
            self.user_pool_info.lp_amount,
            self.user_pool_info.current_lp_amount,
        );
        msg!(
            "user : {} withdraw start : {} , end : {}, deposit : {}, withdraw: {}",
            self.signer.key(),
            self.user_pool_info.deposited_time,
            self.clock.unix_timestamp as u64,
            self.user_pool_info.deposited_amount,
            reward.clone()
        );
        reward
    }
}

impl<'info> TradeIn<'info> {
    pub fn init(&mut self, _way: u8, _amount: u64, _current_price: u64, _decimals: u8) {
        self.trade_info.authority = self.signer.key();
        self.trade_info.way = _way;
        self.trade_info.amount = _amount;
        self.trade_info.entry_price = _current_price;
        self.trade_info.decimals = _decimals;
    }
}
