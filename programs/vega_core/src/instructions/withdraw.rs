use anchor_lang::{
    prelude::*,
    solana_program::{self, entrypoint::ProgramResult},
};
use anchor_spl::token::{Token, TokenAccount};

use crate::{states::*, utils::*};

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
    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,
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

pub fn handler(ctx: Context<Withdraw>) -> ProgramResult {
    let reward: u64 = calc_reward_percent(
        ctx.accounts.user_pool_info.deposited_amount,
        ctx.accounts.user_pool_info.deposited_time,
        ctx.accounts.clock.unix_timestamp as u64,
        ctx.accounts.user_pool_info.lp_amount,
        ctx.accounts.user_pool_info.current_lp_amount,
    );

    let seeds = &[
        b"pool".as_ref(),
        ctx.accounts.config.to_account_info().key.as_ref(),
        ctx.accounts.mint.key.as_ref(),
        &[ctx.accounts.pool.bump],
    ];
    let signer_seeds = &[&seeds[..]];
    transfer_mint_from_vault_to_signer(
        &ctx.accounts.token_program,
        signer_seeds,
        &ctx.accounts.user_ata,
        &ctx.accounts.pool_vault,
        &ctx.accounts.pool,
        reward,
    )?;
    ProgramResult::Ok(())
}
