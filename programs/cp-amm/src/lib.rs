use anchor_lang::prelude::*;

declare_id!("29tsSibY75sYcwXXYkJe1eQxt38F5N6TxQ7Uswrw3A1o");

#[program]
pub mod cp_amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
