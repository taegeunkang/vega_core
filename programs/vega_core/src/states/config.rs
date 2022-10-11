use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub fee_rate: u8,
}

impl Config {

    pub fn init(&mut self, authority : Pubkey , fee_rate : u8 ) -> ProgramResult {
        self.authority = authority;
        self.fee_rate = fee_rate;
        ProgramResult::Ok(())
    }

}