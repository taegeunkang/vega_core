


use anchor_lang::{prelude::*, solana_program::{entrypoint::ProgramResult, native_token::LAMPORTS_PER_SOL, self}};
use anchor_spl::token::{TokenAccount, Token};

use crate::{states::*, utils::*};

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
    #[account(init,seeds=[pool.key().as_ref(), mint.key().as_ref()], bump, payer = signer, token::mint = mint, token::authority = pool)]
    pub pool_vault: Box<Account<'info, TokenAccount>>,
    /// CHECK : this is safe
    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(mut, seeds=[b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(address = solana_program::sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateStakingPool>) -> ProgramResult {
    let pool = &mut ctx.accounts.pool;
    pool.init(ctx.program_id.clone(), ctx.accounts.config.fee_rate, ctx.accounts.mint.key(), ctx.accounts.pool_vault.key(), ctx.accounts.config.key());

    let mint_amount: u64 = 10000000 * LAMPORTS_PER_SOL;
    transfer_mint_from_signer_to_vault(&ctx.accounts.token_program, &ctx.accounts.signer, &ctx.accounts.user_ata, &ctx.accounts.pool_vault, mint_amount)?;
    ProgramResult::Ok(())

}