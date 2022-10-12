use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::states::Pool;
use crate::utils::*;
#[derive(Accounts)]
#[instruction(amount : u64)]
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

pub fn handler(ctx: Context<Sell>, amount: u64) -> ProgramResult {
    // receive vega from signer
    transfer_mint_from_signer_to_vault(
        &ctx.accounts.token_program,
        &ctx.accounts.signer,
        &ctx.accounts.user_ata,
        &ctx.accounts.pool_vault,
        amount,
    )?;
    let sol_amount = amount.checked_div(10).unwrap();
    transfer_sol_from_vault_to_signer(&ctx.accounts.pool, &ctx.accounts.signer, sol_amount)?;

    ProgramResult::Ok(())
}
