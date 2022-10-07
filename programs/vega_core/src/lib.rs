use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token;
use anchor_spl::token::{ Token, TokenAccount, Transfer};
declare_id!("6jMXqfgZoguXohoWktL9UBhAqADYJSZYMC47ENoY2DvW");

mod utils;
mod states;
use utils::*;
use states::*;


#[program]
pub mod vega_core {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        ctx.accounts.config.authority = ctx.accounts.signer.key();
        ctx.accounts.config.fee_rate = 30; // fee amount = amount * fee_rate / 10_000 -> 0.3%
        ctx.accounts.config.lp_mint = ctx.accounts.lp_mint.key();

        msg!("contract is initialized!");

        ProgramResult::Ok(())
    }

    pub fn create_pool(ctx: Context<CreateStakingPool>) -> ProgramResult {
        ctx.accounts.pool.fee_rate = ctx.accounts.config.fee_rate;
        ctx.accounts.pool.mint = ctx.accounts.mint.key();
        ctx.accounts.pool.vault = ctx.accounts.pool_vault.key();
        ctx.accounts.pool.lp_mint = ctx.accounts.lp_mint.key();
        ctx.accounts.pool.lp_supply = 0;
        ctx.accounts.pool.vault_amount = 0;

        let _bump = find_pool_bump(ctx.accounts.config.key(), ctx.accounts.mint.key(), ctx.program_id.clone());
        ctx.accounts.pool.bump = _bump;


        let decimal: u64 = 1000000000;
        let lp_amount: u64 = 100000000 * decimal;
        token::transfer(ctx.accounts.into_transfer_cpi_context_lp(), lp_amount)?;

        ProgramResult::Ok(())
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, _amount_in: u64) -> ProgramResult {
        token::transfer(ctx.accounts.into_transfer_cpi_context_mint(), _amount_in)?;

        let amount_fee = fee_amount(_amount_in, ctx.accounts.pool.fee_rate);
        let _amount = _amount_in.checked_sub(amount_fee).unwrap();
        ctx.accounts.pool.vault_amount = ctx.accounts.pool.vault_amount.checked_add(_amount).unwrap();

        let lp_amount: u64 = calc_lp_amount(ctx.accounts.pool.vault_amount,ctx.accounts.pool.lp_supply,_amount);

        if lp_amount == 0 {
            return ProgramResult::Err(ProgramError::InsufficientFunds);
        }

        ctx.accounts.user_pool_info.authority = ctx.accounts.signer.key();
        ctx.accounts.user_pool_info.pool = ctx.accounts.pool.key();
        ctx.accounts.user_pool_info.mint = ctx.accounts.mint.key();
        ctx.accounts.user_pool_info.deposited_amount = _amount;
        ctx.accounts.user_pool_info.deposited_time = ctx.accounts.clock.unix_timestamp as u64;
        ctx.accounts.user_pool_info.lp_amount = lp_amount;

        let seeds = &[
            b"pool".as_ref(),
            ctx.accounts.config.to_account_info().key.as_ref(),
            ctx.accounts.mint.to_account_info().key.as_ref(),
            &[ctx.accounts.pool.bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_lp_vault.to_account_info(),
                to: ctx.accounts.user_lp_ata.clone(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_context, lp_amount)?;

        ProgramResult::Ok(())
    }
}

#[derive(Accounts)]
#[instruction()]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init, seeds=[b"config"], bump, payer = signer, space = 8 + std::mem::size_of::<Config>())]
    pub config: Account<'info, Config>,
    /// CHECK : this is safe
    pub lp_mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CreateStakingPool<'info> {
    #[account(mut, constraint = signer.key() == config.authority)]
    pub signer: Signer<'info>,
    #[account(init, seeds = [b"pool", config.key().as_ref(), mint.key().as_ref()], bump, payer = signer, space = 8 + std::mem::size_of::<Pool>())]
    pub pool: Box<Account<'info, Pool>>,
    /// CHECK : this is safe
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    /// CHECK : this is safe
    #[account(mut)]
    pub lp_mint: AccountInfo<'info>,
    #[account(init,seeds=[pool.key().as_ref(), mint.key().as_ref()], bump, payer = signer, token::mint = mint, token::authority = pool)]
    pub pool_vault: Box<Account<'info, TokenAccount>>,
    #[account(init, seeds=[pool.key().as_ref(), lp_mint.key().as_ref()], bump, payer = signer, token::mint = lp_mint, token::authority = pool)]
    pub pool_lp_vault: Box<Account<'info, TokenAccount>>,
    /// CHECK : this is safe
    #[account(mut)]
    pub user_lp_ata: AccountInfo<'info>,
    #[account(mut, seeds=[b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(address = solana_program::sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_amount_in : u64)]
pub struct AddLiquidity<'info> {
    /// CHECK : this is safe
    #[account(mut, constraint = authority.key() == config.authority)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, seeds = [b"pool", config.key().as_ref(), mint.key().as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, seeds=[pool.key().as_ref(), mint.key().as_ref()], bump, constraint = pool_vault.owner == pool.key())]
    pub pool_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds=[pool.key().as_ref(), lp_mint.key().as_ref()], bump, constraint = pool_lp_vault.owner == pool.key())]
    pub pool_lp_vault: Box<Account<'info, TokenAccount>>,
    /// CHECK : this is safe
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    /// CHECK : this is safe
    #[account(mut)]
    pub lp_mint: AccountInfo<'info>,
    /// CHECK : this is safe
    #[account(mut)]
    pub user_ata: AccountInfo<'info>,
    /// CHECK : this is safe
    #[account(mut)]
    pub user_lp_ata: AccountInfo<'info>,

    #[account(init, seeds=[b"user_pool_info", signer.key().as_ref(), pool.key().as_ref()], bump, payer = signer, space = 8 + std::mem::size_of::<UserPoolInfo>())]
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

impl<'info> CreateStakingPool<'info> {
    pub fn into_transfer_cpi_context_lp(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_lp_ata.clone(),
                to: self.pool_lp_vault.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }
}

impl<'info> AddLiquidity<'info> {
    pub fn into_transfer_cpi_context_mint(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_ata.to_account_info(),
                to: self.pool_vault.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }

}
