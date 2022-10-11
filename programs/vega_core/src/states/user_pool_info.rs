use anchor_lang::prelude::*;

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

impl UserPoolInfo {
    pub fn init(&mut self, signer : Pubkey , pool : Pubkey, mint : Pubkey, amount : u64, clock : u64,lp_amount : u64) {
        self.authority = signer;
        self.pool = pool;
        self.mint = mint;
        self.deposited_amount = amount;
        self.deposited_time = clock;
        self.lp_amount = lp_amount;
        self.current_lp_amount = lp_amount;
    }
}
