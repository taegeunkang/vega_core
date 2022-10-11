
use anchor_lang::{prelude::*, solana_program::{entrypoint::ProgramResult, self}};
use anchor_spl::token::{TokenAccount, Token};

use crate::{states::*, utils::*};



#[derive(Accounts)]
#[instruction(amount : u64)]
pub struct Deposit<'info> {
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
    pub user_ata: Account<'info, TokenAccount>,
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

pub fn handler(ctx : Context<Deposit>, amount : u64)  -> ProgramResult {

    transfer_mint_from_signer_to_vault(&ctx.accounts.token_program, &ctx.accounts.signer, &ctx.accounts.user_ata, &ctx.accounts.pool_vault , amount)?;

    let amount_fee: u64 = calc_fee_amount(amount, ctx.accounts.pool.fee_rate);
    let amt: u64 = amount.checked_sub(amount_fee).unwrap();
    let vault_amount: u64 = ctx.accounts.pool.vault_amount.checked_add(amt).unwrap();
    let lp_amount: u64 = calc_lp_amount(vault_amount, ctx.accounts.pool.lp_supply, amt);

    if lp_amount == 0 {
        return ProgramResult::Err(ProgramError::InsufficientFunds);
    }
    let user_pool_info  = &mut ctx.accounts.user_pool_info;
    user_pool_info.init(ctx.accounts.signer.key(), ctx.accounts.pool.key(), ctx.accounts.mint.key(), amt, ctx.accounts.clock.unix_timestamp as u64, lp_amount);
    ctx.accounts.pool.vault_amount = vault_amount;
    ProgramResult::Ok(())

}