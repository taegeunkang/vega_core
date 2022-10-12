use crate::{states::*, utils::*};
use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use chainlink_solana as chainlink;

#[derive(Accounts)]
#[instruction()]
pub struct TradeOut<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK : this is safe
    #[account(mut)]
    pub pool: AccountInfo<'info>,
    #[account(mut, seeds=[b"trade", signer.key().as_ref()], bump, close = signer)]
    pub trade_info: Box<Account<'info, TradeInfo>>,
    #[account(mut, seeds=[b"user_pool_info", signer.key().as_ref(), pool.key().as_ref()], bump)]
    pub user_pool_info: Box<Account<'info, UserPoolInfo>>,
    /// CHECK: We're reading data from this specified chainlink feed
    pub chainlink_feed: AccountInfo<'info>,
    /// CHECK: This is the Chainlink program library on Devnet
    pub chainlink_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<TradeOut>) -> ProgramResult {
    let round = chainlink::latest_round_data(
        ctx.accounts.chainlink_program.to_account_info(),
        ctx.accounts.chainlink_feed.to_account_info(),
    )?;

    let current_price = round.answer as u64;

    let (percentage, pl) = calc_pl(
        ctx.accounts.trade_info.entry_price,
        current_price,
        ctx.accounts.trade_info.way,
    );
    let lp_amount = calc_trade_out_lp_amount(ctx.accounts.trade_info.amount, percentage, pl);
    msg!("user trade out");
    msg!("percentage : {}", percentage);
    msg!("entry price : {} , current price : {}", ctx.accounts.trade_info.entry_price, current_price);
    
    msg!(
        "trade in : {}, trade out : {}",
        ctx.accounts.trade_info.amount,
        lp_amount
    );

    ctx.accounts.user_pool_info.current_lp_amount = ctx
        .accounts
        .user_pool_info
        .current_lp_amount
        .checked_add(lp_amount)
        .unwrap();

    ProgramResult::Ok(())
}
