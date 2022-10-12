use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::states::*;

#[derive(Accounts)]
#[instruction(fee_rate : u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init, seeds=[b"config"], bump, payer = signer, space = 8 + std::mem::size_of::<Config>())]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, fee_rate: u8) -> ProgramResult {
    let config = &mut ctx.accounts.config;
    let signer = &ctx.accounts.signer;

    return config.init(signer.key(), fee_rate);
}
