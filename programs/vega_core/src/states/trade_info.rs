use anchor_lang::prelude::*;

#[account]
pub struct TradeInfo {
    pub authority: Pubkey,
    pub way: u8,
    pub entry_price: u64,
    pub decimals: u8,
    pub amount: u64,
}

impl TradeInfo {
    pub fn init(&mut self, signer: Pubkey, way: u8, amount: u64, current_price: u64, decimals: u8) {
        self.authority = signer;
        self.way = way;
        self.amount = amount;
        self.entry_price = current_price;
        self.decimals = decimals;
    }
}
