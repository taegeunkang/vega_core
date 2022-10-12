use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::states::Pool;

pub fn transfer_sol_from_signer_to_vault<'info>(
    system_program: &Program<'info, System>,
    signer: &Signer<'info>,
    vault: &Account<'info, Pool>,
    amount: u64,
) -> Result<()> {
    system_program::transfer(
        CpiContext::new(
            system_program.to_account_info(),
            system_program::Transfer {
                from: signer.to_account_info(),
                to: vault.to_account_info(),
            },
        ),
        amount,
    )
}

pub fn transfer_mint_from_vault_to_signer<'info>(
    token_program: &Program<'info, Token>,
    signer_seeds: &[&[&[u8]]; 1],
    signer_ata: &Account<'info, TokenAccount>,
    vault: &Account<'info, TokenAccount>,
    pool: &Account<'info, Pool>,
    amount: u64,
) -> Result<()> {
    token::transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: vault.to_account_info(),
                to: signer_ata.to_account_info(),
                authority: pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )
}

pub fn transfer_mint_from_signer_to_vault<'info>(
    token_program: &Program<'info, Token>,
    signer: &Signer<'info>,
    signer_ata: &Account<'info, TokenAccount>,
    vault: &Account<'info, TokenAccount>,
    amount: u64,
) -> Result<()> {
    token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: signer_ata.to_account_info(),
                to: vault.to_account_info(),
                authority: signer.to_account_info(),
            },
        ),
        amount,
    )
}

pub fn transfer_sol_from_vault_to_signer<'info>(
    vault: &Account<'info, Pool>,
    signer: &Signer<'info>,
    amount: u64,
) -> Result<()> {
    **vault.to_account_info().try_borrow_mut_lamports()? -= amount;
    **signer.to_account_info().try_borrow_mut_lamports()? += amount;
    Ok(())
}
