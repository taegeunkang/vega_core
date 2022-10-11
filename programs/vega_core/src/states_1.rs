use anchor_lang::prelude::*;



#[account]
pub struct TradeInfo {
    pub authority: Pubkey,
    pub way: u8,
    pub entry_price: u64,
    pub decimals: u8,
    pub amount: u64,
}
