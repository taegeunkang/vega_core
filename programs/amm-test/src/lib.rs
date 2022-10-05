use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer, MintTo};
declare_id!("5iJ1KCrWEQpR4FDMJrQWMdLFv3HdBF45kpzabkghmBPx");

mod states;
use states::*;

#[program]
pub mod amm_test {

    use anchor_spl::token::accessor::amount;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        ctx.accounts.config.authority = ctx.accounts.signer.key();
        ctx.accounts.config.fee_rate = 30; // fee amount = amount * fee_rate / 10_000
        ctx.accounts.config.lp_mint = ctx.accounts.lp_mint.key();

        msg!("contract is initialized!");

        ProgramResult::Ok(())
    }

    pub fn create_pool(ctx: Context<CreatePool>, _pool_pair: String) -> ProgramResult {
        ctx.accounts.pool.fee_rate = ctx.accounts.config.fee_rate;
        ctx.accounts.pool.mint_a = ctx.accounts.mint_a.key();
        ctx.accounts.pool.mint_b = ctx.accounts.mint_b.key();
        ctx.accounts.pool.vault_a = ctx.accounts.pool_vault_a.key();
        ctx.accounts.pool.vault_b = ctx.accounts.pool_vault_b.key();
        ctx.accounts.pool.lp_mint = ctx.accounts.lp_mint.key();
        ctx.accounts.pool.amount_a = 0;
        ctx.accounts.pool.amount_b = 0;
        ctx.accounts.pool.reward_last_updated_timestamp = 0;
        ctx.accounts.pool.reward_mint = ctx.accounts.reward_mint.key();

        msg!(
            "pool created! mint_a : {} mint_b : {} reward_token : {}",
            ctx.accounts.mint_a.key(),
            ctx.accounts.mint_b.key(),
            ctx.accounts.reward_mint.key()
        );
        ProgramResult::Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        _amount_a: u64,
        _amount_b: u64,
    ) -> ProgramResult {
        let balance_a : u64 = ctx.accounts.pool_vault_a.amount;
        let balance_b : u64 = ctx.accounts.pool_vault_b.amount;
        token::transfer(ctx.accounts.into_transfer_cpi_context_mint_a(), _amount_a)?;
        token::transfer(ctx.accounts.into_transfer_cpi_context_mint_b(), _amount_b)?;
        // token::mint_to(ctx, amount)
        let mut amount_a_in :u64 = ctx.accounts.pool_vault_a.amount.checked_sub(balance_a).unwrap();
        let mut amount_b_in : u64 = ctx.accounts.pool_vault_b.amount.checked_sub(balance_b).unwrap();
        amount_a_in = amount_a_in.checked_sub(amount_a_in.checked_mul(ctx.accounts.pool.fee_rate as u64).unwrap()).unwrap();
        amount_b_in = amount_b_in.checked_sub(amount_b_in.checked_mul(ctx.accounts.pool.fee_rate as u64).unwrap()).unwrap();


        ctx.accounts.pool.amount_a = ctx.accounts.pool.amount_a.checked_add(amount_a_in).unwrap();
        ctx.accounts.pool.amount_b = ctx.accounts.pool.amount_b.checked_add(amount_b_in).unwrap();



        let lp_amount : u64 = ctx.accounts.calc_lp_amount(amount_a_in, amount_b_in);

        if lp_amount == 0 {
            return ProgramResult::Err(ProgramError::InsufficientFunds);
        }

        ctx.accounts.user_pool_info.authority = ctx.accounts.signer.key();
        ctx.accounts.user_pool_info.pool = ctx.accounts.pool.key();
        ctx.accounts.user_pool_info.mint_a = ctx.accounts.mint_a.key();
        ctx.accounts.user_pool_info.mint_b = ctx.accounts.mint_b.key();
        ctx.accounts.user_pool_info.amount_a = amount_a_in;
        ctx.accounts.user_pool_info.amount_b = amount_b_in;
        ctx.accounts.user_pool_info.last_deposited_time = ctx.accounts.clock.unix_timestamp as u64;


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
    pub lp_mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CreatePool<'info> {
    #[account(mut, constraint = signer.key() == config.authority)]
    pub signer: Signer<'info>,
    #[account(init, seeds = [b"pool", config.key().as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref()], bump, payer = signer, space = 8 + std::mem::size_of::<Pool>())]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut)]
    pub mint_a: Account<'info, Mint>,
    #[account(mut)]
    pub mint_b: Account<'info, Mint>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    #[account(init, payer = signer, token::mint = mint_a, token::authority = pool)]
    pub pool_vault_a: Box<Account<'info, TokenAccount>>,
    #[account(init, payer = signer, token::mint = mint_b, token::authority = pool)]
    pub pool_vault_b: Box<Account<'info, TokenAccount>>,
    pub reward_mint: AccountInfo<'info>,
    #[account(mut, seeds=[b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(address = solana_program::sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_amount_a : u64, _amount_b : u64)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, seeds = [b"pool", config.key().as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, token::mint = mint_a, token::authority = pool)]
    pub pool_vault_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = pool)]
    pub pool_vault_b: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub mint_a: Account<'info, Mint>,
    #[account(mut)]
    pub mint_b: Account<'info, Mint>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    #[account(mut, token::mint = mint_a, token::authority = signer)]
    pub user_a_ata: Account<'info, TokenAccount>,
    #[account(mut, token::mint = mint_b, token::authority = signer)]
    pub user_b_ata: Account<'info, TokenAccount>,

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

impl<'info> AddLiquidity<'info> {
    pub fn into_transfer_cpi_context_mint_a(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let transfer = Transfer {
            from: self.user_a_ata.to_account_info(),
            to: self.pool_vault_a.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        CpiContext::new(self.token_program.to_account_info(), transfer)
    }
    pub fn into_transfer_cpi_context_mint_b(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let transfer = Transfer {
            from: self.user_b_ata.to_account_info(),
            to: self.pool_vault_b.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        CpiContext::new(self.token_program.to_account_info(), transfer)
    }

    // pub fn into_mint_to_cpi_context_lp(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
    //     let mint_to = MintTo {
    //         mint : self.lp_mint.to_account_info(),
    //         to : self.signer.to_account_info(),

    //     }
    // }

    pub fn calc_lp_amount(&self, amount_a_in: u64, amount_b_in: u64) -> u64 {
        let pool_a_amount: u64 = self.pool.amount_a;
        let pool_b_amount: u64 = self.pool.amount_b;
        let total_supply: u64 = self.lp_mint.supply;
        let mut liquidity: u64 = 0;

        if total_supply == 0 {
            liquidity = sqrt(amount_a_in.checked_mul(amount_b_in).unwrap())
                .checked_sub(1000)
                .unwrap();
            //1000개 기본 락
        } else {
            liquidity = min(
                amount_a_in
                    .checked_mul(total_supply)
                    .unwrap()
                    .checked_div(pool_a_amount)
                    .unwrap(),
                amount_b_in
                    .checked_mul(total_supply)
                    .unwrap()
                    .checked_div(pool_b_amount)
                    .unwrap(),
            );
        }

        liquidity
    }
}

pub fn sqrt(mut y: u64) -> u64 {
    let mut z: u64 = 0;

    if y > 3 {
        z = y;
        let mut x = y.checked_div(2).unwrap().checked_add(1).unwrap();

        loop {
            z = x;
            x = y
                .checked_div(x)
                .unwrap()
                .checked_add(x)
                .unwrap()
                .checked_div(2)
                .unwrap();

            if x >= z {
                break;
            }
        }
    } else if y != 0 {
        z = 1;
    }

    z
}

pub fn min(_a: u64, _b: u64) -> u64 {
    let mn = if _a > _b { _b } else { _a };
    mn
}
