use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub fee_rate: u16,
    pub lp_mint: Pubkey,
}

#[account]
pub struct Pool {
    pub fee_rate: u16,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub vault_amount: u64,
    pub lp_mint: Pubkey,
    pub lp_supply: u64,
    pub bump: u8,
}

#[account]
pub struct UserPoolInfo {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub deposited_amount: u64,
    pub deposited_time: u64,
    pub lp_amount: u64,
    pub current_lp_amount: u64,
}

#[account]
pub struct TradeInfo {
    pub authority: Pubkey,
    pub way: u8,
    pub entry_price: u64,
    pub decimals: u8,
    pub amount: u64,
}
