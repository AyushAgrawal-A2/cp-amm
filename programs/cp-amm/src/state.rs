use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LiquidityPoolConfig {
    pub vault_a: u64,
    pub vault_b: u64,
    pub fee: u16,
    pub liquidity_pool_mint_bump: u8,
    pub bump: u8,
}
