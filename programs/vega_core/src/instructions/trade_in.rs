use crate::states::*;
use anchor_lang::{
    prelude::*,
    solana_program::{entrypoint::ProgramResult},
};
use chainlink_solana as chainlink;

#[derive(Accounts)]
#[instruction(way : u8, amount : u64)]
pub struct TradeIn<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK : this is safe
    #[account(mut)]
    pub pool: AccountInfo<'info>,
    #[account(init, seeds=[b"trade", signer.key().as_ref()], bump, payer = signer, space= 8 + std::mem::size_of::<TradeInfo>())]
    pub trade_info: Box<Account<'info, TradeInfo>>,
    #[account(mut, seeds=[b"user_pool_info", signer.key().as_ref(), pool.key().as_ref()], bump)]
    pub user_pool_info: Box<Account<'info, UserPoolInfo>>,
    /// CHECK: We're reading data from this specified chainlink feed
    pub chainlink_feed: AccountInfo<'info>,
    /// CHECK: This is the Chainlink program library on Devnet
    pub chainlink_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<TradeIn>, way: u8, amount: u64) -> ProgramResult {
    let round = chainlink::latest_round_data(
        ctx.accounts.chainlink_program.clone(),
        ctx.accounts.chainlink_feed.clone(),
    )?;

    let decimals = chainlink::decimals(
        ctx.accounts.chainlink_program.clone(),
        ctx.accounts.chainlink_feed.clone(),
    )?;
    let current_price = round.answer as u64;
    msg!("current price : {}, decimals : {}", current_price, decimals);
    msg!(
        "trade_in way : {}, current_price : {}, decimals : {}, amount : {}",
        way,
        current_price,
        decimals,
        amount
    );

    let trade_info = &mut ctx.accounts.trade_info;
    trade_info.init(
        ctx.accounts.signer.key(),
        way,
        amount,
        current_price,
        decimals,
    );
    ctx.accounts.user_pool_info.current_lp_amount = ctx
        .accounts
        .user_pool_info
        .current_lp_amount
        .checked_sub(amount)
        .unwrap();

    ProgramResult::Ok(())
}
