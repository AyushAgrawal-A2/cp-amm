use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LiquidityPoolConfig {
    pub fee: u16,
    pub liquidity_pool_mint_bump: u8,
    pub bump: u8,
}
