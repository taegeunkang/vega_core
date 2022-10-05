use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub fee_rate: u16,
    pub lp_mint : Pubkey,
}

#[account]
pub struct Pool {
    pub fee_rate: u16,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub amount_a : u64,
    pub amount_b : u64,
    pub lp_mint : Pubkey,
    pub reward_last_updated_timestamp: u64,
    pub reward_mint: Pubkey,
}

#[account]
pub struct UserPoolInfo {
    pub authority: Pubkey,
    pub pool : Pubkey,
    pub mint_a : Pubkey,
    pub mint_b : Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub last_deposited_time: u64,
    pub lp_amount : u64,
}
