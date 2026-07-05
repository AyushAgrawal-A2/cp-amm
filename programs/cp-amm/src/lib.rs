pub mod constants;
pub mod cp_math;
pub mod error;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use cp_math::*;
pub use error::*;
pub use instructions::*;
pub use state::*;

declare_id!("XRv9BHseB1DJkudJ7ZHew1ahSNXmQCs2k1HUPTQgkHE");

#[program]
pub mod cp_amm {
    use super::*;

    pub fn create_pool(ctx: Context<CreatePool>, fee: u16) -> Result<()> {
        handle_create_pool(ctx, fee)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a_desired: u64,
        amount_b_desired: u64,
    ) -> Result<()> {
        handle_add_liquidity(ctx, amount_a_desired, amount_b_desired)
    }

    pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount_lp: u64) -> Result<()> {
        handle_withdraw_liquidity(ctx, amount_lp)
    }
}
