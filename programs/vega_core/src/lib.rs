use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::system_program;
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint};
use anchor_spl::token::{Token, TokenAccount, Transfer};
declare_id!("Cbvd322Xp7Qfkf2VvgyCH4cKkJXwypDYsGK8upAm3ZXK");

mod states;
mod utils;
use states::*;
use utils::*;

#[program]
pub mod vega_core {

    use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        ctx.accounts.init();
        msg!("contract is initialized!");

        ProgramResult::Ok(())
    }

    pub fn buy(ctx: Context<Buy>, _amount: u64) -> ProgramResult {
        // receive sol from signer
        system_program::transfer(ctx.accounts.into_transfer_cpi_context_sol(), _amount)?;

        let seeds = &[
            b"pool".as_ref(),
            ctx.accounts.config.key.as_ref(),
            ctx.accounts.mint.to_account_info().key.as_ref(),
            &[ctx.accounts.pool.bump],
        ];
        let signer = &[&seeds[..]];

        let vega_amount = _amount.checked_mul(10).unwrap();
        // transfer vega to signer
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_vault.to_account_info(),
                to: ctx.accounts.user_ata.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_context, vega_amount)?;

        ProgramResult::Ok(())
    }

    pub fn sell(ctx: Context<Sell>, _amount: u64) -> ProgramResult {
        // receive vega from signer
        token::transfer(ctx.accounts.into_transfer_cpi_context_mint(), _amount)?;
        let sol_amount = _amount.checked_div(10).unwrap();

        **ctx
            .accounts
            .pool
            .to_account_info()
            .try_borrow_mut_lamports()? -= sol_amount;
        **ctx
            .accounts
            .signer
            .to_account_info()
            .try_borrow_mut_lamports()? += sol_amount;
        ProgramResult::Ok(())
    }

    pub fn create_pool(ctx: Context<CreateStakingPool>) -> ProgramResult {
        ctx.accounts.init(ctx.program_id.clone());

        let mint_amount: u64 = 10000000 * LAMPORTS_PER_SOL;
        let lp_amount: u64 = 100000000 * LAMPORTS_PER_SOL;
        token::transfer(ctx.accounts.into_transfer_cpi_context_mint(), mint_amount)?;
        token::transfer(ctx.accounts.into_transfer_cpi_context_lp(), lp_amount)?;

        ProgramResult::Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, _amount_in: u64) -> ProgramResult {
        token::transfer(ctx.accounts.into_transfer_cpi_context_mint(), _amount_in)?;

        let amount_fee: u64 = fee_amount(_amount_in, ctx.accounts.pool.fee_rate);
        let amount: u64 = _amount_in.checked_sub(amount_fee).unwrap();
        let vault_amount: u64 = ctx.accounts.pool.vault_amount.checked_add(amount).unwrap();
        let lp_amount: u64 = calc_lp_amount(vault_amount, ctx.accounts.pool.lp_supply, amount);

        if lp_amount == 0 {
            return ProgramResult::Err(ProgramError::InsufficientFunds);
        }

        ctx.accounts.init_user_pool_info(amount, lp_amount);

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
                from: ctx.accounts.pool_lp_vault.to_account_info(),
                to: ctx.accounts.user_lp_ata.clone(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_context, lp_amount)?;

        ProgramResult::Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> ProgramResult {
        let reward: u64 = ctx.accounts.calc_reward();

        token::transfer(
            ctx.accounts.into_transfer_cpi_context_lp(),
            ctx.accounts.user_lp_ata.amount,
        )?;

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
#[instruction(_amount : u64)]
pub struct Buy<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut, constraint = user_ata.owner == signer.key())]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"pool", config.key().as_ref(), mint.key().as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut,seeds=[pool.key().as_ref(), mint.key().as_ref()], bump, constraint = pool_vault.owner == pool.key())]
    pub pool_vault: Box<Account<'info, TokenAccount>>,
    /// CHECK : this is safe
    pub config: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct Sell<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut, constraint = user_ata.owner == signer.key())]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"pool", config.key().as_ref(), mint.key().as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut,seeds=[pool.key().as_ref(), mint.key().as_ref()], bump, constraint = pool_vault.owner == pool.key())]
    pub pool_vault: Box<Account<'info, TokenAccount>>,
    /// CHECK : this is safe
    pub config: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
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
    /// CHECK : this is safe
    #[account(mut)]
    pub user_ata: AccountInfo<'info>,
    #[account(mut, seeds=[b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(address = solana_program::sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_amount_in : u64)]
pub struct Deposit<'info> {
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

#[derive(Accounts)]
#[instruction()]
pub struct Withdraw<'info> {
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
    #[account(mut, token::mint = lp_mint, token::authority = signer)]
    pub user_lp_ata: Account<'info, TokenAccount>,
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

impl<'info> Initialize<'info> {
    pub fn init(&mut self) {
        self.config.authority = self.signer.key();
        self.config.fee_rate = 30; // fee amount = amount * fee_rate / 10_000 -> 0.3%
        self.config.lp_mint = self.lp_mint.key();
    }
}

impl<'info> Buy<'info> {
    pub fn into_transfer_cpi_context_sol(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, system_program::Transfer<'info>> {
        CpiContext::new(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: self.signer.to_account_info(),
                to: self.pool.to_account_info(),
            },
        )
    }
}

impl<'info> Sell<'info> {
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

impl<'info> CreateStakingPool<'info> {
    pub fn init(&mut self, _program_id: Pubkey) {
        self.pool.fee_rate = self.config.fee_rate;
        self.pool.mint = self.mint.key();
        self.pool.vault = self.pool_vault.key();
        self.pool.lp_mint = self.lp_mint.key();
        self.pool.lp_supply = 0;
        self.pool.vault_amount = 0;

        let _bump = find_pool_bump(self.config.key(), self.mint.key(), _program_id);
        self.pool.bump = _bump;
    }
    pub fn into_transfer_cpi_context_mint(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_ata.clone(),
                to: self.pool_vault.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }

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

impl<'info> Deposit<'info> {
    pub fn init_user_pool_info(&mut self, _amount: u64, _lp_amount: u64) {
        self.user_pool_info.authority = self.signer.key();
        self.user_pool_info.pool = self.pool.key();
        self.user_pool_info.mint = self.mint.key();
        self.user_pool_info.deposited_amount = _amount;
        self.user_pool_info.deposited_time = self.clock.unix_timestamp as u64;
        self.user_pool_info.lp_amount = _lp_amount;
        self.pool.vault_amount = self.pool.vault_amount.checked_add(_amount).unwrap();
    }

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

impl<'info> Withdraw<'info> {
    pub fn into_transfer_cpi_context_lp(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_lp_ata.to_account_info(),
                to: self.pool_lp_vault.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }

    pub fn calc_reward(&self) -> u64 {
        let reward: u64 = calc_reward_percent(
            self.user_pool_info.deposited_amount,
            self.user_pool_info.deposited_time,
            self.clock.unix_timestamp as u64,
            self.user_pool_info.lp_amount,
            self.user_lp_ata.amount,
        );
        msg!(
            "user : {} withdraw start : {} , end : {}, deposit : {}, reward : {}",
            self.signer.key(),
            self.user_pool_info.deposited_time,
            self.clock.unix_timestamp as u64,
            self.user_pool_info.deposited_amount,
            reward.clone()
        );
        reward
    }
}
