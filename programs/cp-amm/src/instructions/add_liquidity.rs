use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{
    add_liquidity_amount, AMMError, LiquidityPoolConfig, LIQUIDITY_POOL_MINT_SEED,
    LIQUIDITY_POOL_SEED,
};

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
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
        mint::authority = liquidity_pool_config,
        mint::token_program = token_program_liquidity_pool_mint,
        seeds = [LIQUIDITY_POOL_MINT_SEED, liquidity_pool_config.key().as_ref()],
        bump = liquidity_pool_config.liquidity_pool_mint_bump
    )]
    pub liquidity_pool_mint: Box<InterfaceAccount<'info, Mint>>,

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

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = liquidity_pool_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program_liquidity_pool_mint,
    )]
    pub payer_liquidity_pool_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program_mint_a: Interface<'info, TokenInterface>,
    pub token_program_mint_b: Interface<'info, TokenInterface>,
    pub token_program_liquidity_pool_mint: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handle_add_liquidity(
    ctx: Context<AddLiquidity>,
    amount_a_desired: u64,
    amount_b_desired: u64,
) -> Result<()> {
    require!(
        amount_a_desired > 0 && amount_b_desired > 0,
        AMMError::InvalidArguments
    );

    let (amount_a, amount_b, amount_lp) = add_liquidity_amount(
        amount_a_desired,
        amount_b_desired,
        ctx.accounts.liquidity_pool_vault_a.amount,
        ctx.accounts.liquidity_pool_vault_b.amount,
        ctx.accounts.liquidity_pool_mint.supply,
    )?;
    require!(
        amount_a > 0 && amount_b > 0 && amount_lp > 0,
        AMMError::InvalidAmount
    );

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program_mint_a.key(),
            token_interface::TransferChecked {
                from: ctx.accounts.payer_mint_a_ata.to_account_info(),
                mint: ctx.accounts.mint_a.to_account_info(),
                to: ctx.accounts.liquidity_pool_vault_a.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount_a,
        ctx.accounts.mint_a.decimals,
    )?;
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program_mint_b.key(),
            token_interface::TransferChecked {
                from: ctx.accounts.payer_mint_b_ata.to_account_info(),
                mint: ctx.accounts.mint_b.to_account_info(),
                to: ctx.accounts.liquidity_pool_vault_b.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount_b,
        ctx.accounts.mint_b.decimals,
    )?;
    let mint_a_address = ctx.accounts.mint_a.key();
    let mint_b_address = ctx.accounts.mint_b.key();
    let liquidity_pool_config_seeds = [
        LIQUIDITY_POOL_SEED,
        mint_a_address.as_ref(),
        mint_b_address.as_ref(),
        &[ctx.accounts.liquidity_pool_config.bump],
    ];
    token_interface::mint_to_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program_liquidity_pool_mint.key(),
            token_interface::MintToChecked {
                mint: ctx.accounts.liquidity_pool_mint.to_account_info(),
                to: ctx.accounts.payer_liquidity_pool_ata.to_account_info(),
                authority: ctx.accounts.liquidity_pool_config.to_account_info(),
            },
            &[&liquidity_pool_config_seeds],
        ),
        amount_lp,
        ctx.accounts.liquidity_pool_mint.decimals,
    )?;
    Ok(())
}
