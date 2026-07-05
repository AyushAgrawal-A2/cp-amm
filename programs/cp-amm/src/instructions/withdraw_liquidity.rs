use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{
    withdraw_liquidity_amount, AMMError, LiquidityPoolConfig, LIQUIDITY_POOL_MINT_SEED,
    LIQUIDITY_POOL_SEED,
};

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
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
        mut,
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
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_a,
        associated_token::authority = payer,
        associated_token::token_program = token_program_mint_a,
    )]
    pub payer_mint_a_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_b,
        associated_token::authority = payer,
        associated_token::token_program = token_program_mint_b,
    )]
    pub payer_mint_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
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

pub fn handle_withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount_lp: u64) -> Result<()> {
    require!(amount_lp > 0, AMMError::InvalidArguments);

    let (amount_a, amount_b) = withdraw_liquidity_amount(
        amount_lp,
        ctx.accounts.liquidity_pool_config.vault_a,
        ctx.accounts.liquidity_pool_config.vault_b,
        ctx.accounts.liquidity_pool_mint.supply,
    )?;
    require!(
        amount_a <= ctx.accounts.liquidity_pool_config.vault_a
            && amount_b <= ctx.accounts.liquidity_pool_config.vault_b,
        AMMError::InvalidAmount
    );
    ctx.accounts.liquidity_pool_config.vault_a = ctx
        .accounts
        .liquidity_pool_config
        .vault_a
        .checked_sub(amount_a)
        .ok_or(AMMError::Underflow)?;
    ctx.accounts.liquidity_pool_config.vault_b = ctx
        .accounts
        .liquidity_pool_config
        .vault_b
        .checked_sub(amount_b)
        .ok_or(AMMError::Underflow)?;

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
            ctx.accounts.token_program_mint_a.key(),
            token_interface::TransferChecked {
                from: ctx.accounts.liquidity_pool_vault_a.to_account_info(),
                mint: ctx.accounts.mint_a.to_account_info(),
                to: ctx.accounts.payer_mint_a_ata.to_account_info(),
                authority: ctx.accounts.liquidity_pool_config.to_account_info(),
            },
            &[&liquidity_pool_config_seeds],
        ),
        amount_a,
        ctx.accounts.mint_a.decimals,
    )?;
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program_mint_b.key(),
            token_interface::TransferChecked {
                from: ctx.accounts.liquidity_pool_vault_b.to_account_info(),
                mint: ctx.accounts.mint_b.to_account_info(),
                to: ctx.accounts.payer_mint_b_ata.to_account_info(),
                authority: ctx.accounts.liquidity_pool_config.to_account_info(),
            },
            &[&liquidity_pool_config_seeds],
        ),
        amount_b,
        ctx.accounts.mint_b.decimals,
    )?;

    token_interface::burn_checked(
        CpiContext::new(
            ctx.accounts.token_program_liquidity_pool_mint.key(),
            token_interface::BurnChecked {
                mint: ctx.accounts.liquidity_pool_mint.to_account_info(),
                from: ctx.accounts.payer_liquidity_pool_ata.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount_lp,
        ctx.accounts.liquidity_pool_mint.decimals,
    )?;
    Ok(())
}
