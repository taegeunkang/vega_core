use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{Mint, Token, TokenAccount};
declare_id!("5iJ1KCrWEQpR4FDMJrQWMdLFv3HdBF45kpzabkghmBPx");

#[program]
pub mod amm_test {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        ProgramResult::Ok(())
    }

    pub fn create_ata(ctx: Context<CreateATA>) -> ProgramResult {
        ProgramResult::Ok(())
    }
}

#[derive(Accounts)]
#[instruction()]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init, seeds = [b"counter"], bump, payer = signer, space = 8 + std::mem::size_of::<MintCounter>())]
    pub counter: Account<'info, MintCounter>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateATA<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init, seeds = [mint.key().as_ref(), &[counter.count]], bump, payer = signer ,token::mint = mint, token::authority = signer)]
    pub pool: Box<Account<'info, TokenAccount>>,
    pub mint: Account<'info, Mint>,
    #[account(mut, seeds=[b"counter"], bump)]
    pub counter: Account<'info, MintCounter>,
    #[account(address = solana_program::sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct MintCounter {
    pub count: u8,
}
