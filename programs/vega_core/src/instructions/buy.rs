

use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_spl::token::{TokenAccount, Mint, Token};

use crate::{states::*, utils::*};

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

pub fn handler(ctx : Context<Buy>,  amount: u64) -> ProgramResult {
    // receive sol from signer  
    
    transfer_sol_from_signer_to_vault(&ctx.accounts.system_program, &ctx.accounts.signer, &ctx.accounts.pool, amount)?;

    let seeds = &[
        b"pool".as_ref(),
        ctx.accounts.config.key.as_ref(),
        ctx.accounts.mint.to_account_info().key.as_ref(),
        &[ctx.accounts.pool.bump],
    ];
    let signer_seeds = &[&seeds[..]];
    let vega_amount = amount.checked_mul(10).unwrap();
    transfer_mint_from_vault_to_signer(&ctx.accounts.token_program, signer_seeds, &ctx.accounts.user_ata, &ctx.accounts.pool_vault, &ctx.accounts.pool, vega_amount)?;


    return ProgramResult::Ok(());


}