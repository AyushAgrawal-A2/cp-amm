use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    AMMError, LiquidityPoolConfig, BASIS_POINTS, LIQUIDITY_POOL_MINT_SEED, LIQUIDITY_POOL_SEED,
};

#[derive(Accounts)]
pub struct CreatePool<'info> {
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
        init,
        payer = payer,
        space = 8 + LiquidityPoolConfig::INIT_SPACE,
        seeds = [LIQUIDITY_POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub liquidity_pool_config: Box<Account<'info, LiquidityPoolConfig>>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 9,
        mint::authority = liquidity_pool_config,
        mint::token_program = token_program_liquidity_pool_mint,
        seeds = [LIQUIDITY_POOL_MINT_SEED, liquidity_pool_config.key().as_ref()],
        bump
    )]
    pub liquidity_pool_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_a,
        associated_token::authority = liquidity_pool_config,
        associated_token::token_program = token_program_mint_a,
    )]
    pub liquidity_pool_vault_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_b,
        associated_token::authority = liquidity_pool_config,
        associated_token::token_program = token_program_mint_b,
    )]
    pub liquidity_pool_vault_b: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program_mint_a: Interface<'info, TokenInterface>,
    pub token_program_mint_b: Interface<'info, TokenInterface>,
    pub token_program_liquidity_pool_mint: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_pool(ctx: Context<CreatePool>, fee: u16) -> Result<()> {
    require!(
        ctx.accounts.mint_a.key() < ctx.accounts.mint_b.key(),
        AMMError::InvalidArguments
    );
    require!((fee as u64) < BASIS_POINTS, AMMError::InvalidArguments);
    ctx.accounts
        .liquidity_pool_config
        .set_inner(LiquidityPoolConfig {
            vault_a: 0,
            vault_b: 0,
            fee,
            liquidity_pool_mint_bump: ctx.bumps.liquidity_pool_mint,
            bump: ctx.bumps.liquidity_pool_config,
        });
    Ok(())
}
