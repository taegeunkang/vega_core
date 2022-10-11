use anchor_lang::prelude::*;


use crate::utils::*;
#[account]
pub struct Pool {
    pub fee_rate: u8,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub vault_amount: u64,
    pub lp_mint: Pubkey,
    pub lp_supply: u64,
    pub bump: u8,
}


impl Pool {
    pub fn init(&mut self, program_id: Pubkey, fee_rate : u8, mint : Pubkey, pool_vault : Pubkey, config: Pubkey ) {
        self.fee_rate = fee_rate;
        self.mint = mint;
        self.vault = pool_vault;
        self.lp_supply = 0;
        self.vault_amount = 0;
        self.bump = find_pool_bump(config, mint, program_id);
    }
}