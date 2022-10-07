use anchor_lang::prelude::Pubkey;


pub fn find_pool_bump(_config : Pubkey, _mint : Pubkey, _program_id : Pubkey) -> u8 {
    let seeds = &[
        b"pool",
        _config.as_ref(),
        _mint.as_ref(),
    ];
    let (_pda, _bump) = Pubkey::find_program_address(seeds, &_program_id);
    _bump
}