use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::{swap_amount, AMMError, LiquidityPoolConfig, LIQUIDITY_POOL_SEED};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mint::token_program = token_program_mint_a
    )]
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mint::token_program = token_program_mint_b
    )]
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [LIQUIDITY_POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump = liquidity_pool_config.bump
    )]
    pub liquidity_pool_config: Box<Account<'info, LiquidityPoolConfig>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = liquidity_pool_config,
        associated_token::token_program = token_program_mint_a,
    )]
    pub liquidity_pool_vault_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = liquidity_pool_config,
        associated_token::token_program = token_program_mint_b,
    )]
    pub liquidity_pool_vault_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = payer,
        associated_token::token_program = token_program_mint_a,
    )]
    pub payer_mint_a_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = payer,
        associated_token::token_program = token_program_mint_b,
    )]
    pub payer_mint_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program_mint_a: Interface<'info, TokenInterface>,
    pub token_program_mint_b: Interface<'info, TokenInterface>,
}

pub fn handle_swap(
    ctx: Context<Swap>,
    amount_a_in: u64,
    amount_b_in: u64,
    min_amount_out: u64,
) -> Result<()> {
    require!(
        amount_a_in == 0 || amount_b_in == 0,
        AMMError::InvalidArguments
    );
    require!(
        amount_a_in > 0 || amount_b_in > 0,
        AMMError::InvalidArguments
    );

    let (
        amount_in,
        decimal_in,
        mint_in,
        payer_ata_in,
        vault_ata_in,
        token_program_in,
        amount_out,
        decimal_out,
        mint_out,
        payer_ata_out,
        vault_ata_out,
        token_program_out,
    ) = if amount_a_in == 0 {
        let amount_a_out = swap_amount(
            amount_b_in,
            ctx.accounts.liquidity_pool_vault_b.amount,
            ctx.accounts.liquidity_pool_vault_a.amount,
            ctx.accounts.liquidity_pool_config.fee,
        )?;
        (
            amount_b_in,
            ctx.accounts.mint_b.decimals,
            ctx.accounts.mint_b.to_account_info(),
            ctx.accounts.payer_mint_b_ata.to_account_info(),
            ctx.accounts.liquidity_pool_vault_b.to_account_info(),
            ctx.accounts.token_program_mint_b.key(),
            amount_a_out,
            ctx.accounts.mint_a.decimals,
            ctx.accounts.mint_a.to_account_info(),
            ctx.accounts.payer_mint_a_ata.to_account_info(),
            ctx.accounts.liquidity_pool_vault_a.to_account_info(),
            ctx.accounts.token_program_mint_a.key(),
        )
    } else {
        let amount_b_out = swap_amount(
            amount_a_in,
            ctx.accounts.liquidity_pool_vault_a.amount,
            ctx.accounts.liquidity_pool_vault_b.amount,
            ctx.accounts.liquidity_pool_config.fee,
        )?;
        (
            amount_a_in,
            ctx.accounts.mint_a.decimals,
            ctx.accounts.mint_a.to_account_info(),
            ctx.accounts.payer_mint_a_ata.to_account_info(),
            ctx.accounts.liquidity_pool_vault_a.to_account_info(),
            ctx.accounts.token_program_mint_a.key(),
            amount_b_out,
            ctx.accounts.mint_b.decimals,
            ctx.accounts.mint_b.to_account_info(),
            ctx.accounts.payer_mint_b_ata.to_account_info(),
            ctx.accounts.liquidity_pool_vault_b.to_account_info(),
            ctx.accounts.token_program_mint_b.key(),
        )
    };
    require!(
        amount_in > 0 && amount_out > min_amount_out,
        AMMError::InvalidAmount
    );

    token_interface::transfer_checked(
        CpiContext::new(
            token_program_in,
            token_interface::TransferChecked {
                from: payer_ata_in,
                mint: mint_in,
                to: vault_ata_in,
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount_in,
        decimal_in,
    )?;

    let mint_a_address = ctx.accounts.mint_a.key();
    let mint_b_address = ctx.accounts.mint_b.key();
    let liquidity_pool_config_seeds = [
        LIQUIDITY_POOL_SEED,
        mint_a_address.as_ref(),
        mint_b_address.as_ref(),
        &[ctx.accounts.liquidity_pool_config.bump],
    ];
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            token_program_out,
            token_interface::TransferChecked {
                from: vault_ata_out,
                mint: mint_out,
                to: payer_ata_out,
                authority: ctx.accounts.liquidity_pool_config.to_account_info(),
            },
            &[&liquidity_pool_config_seeds],
        ),
        amount_out,
        decimal_out,
    )?;
    Ok(())
}
