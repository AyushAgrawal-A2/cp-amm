pub use anchor_lang::prelude::*;

#[error_code]
pub enum AMMError {
    #[msg("Invalid arguments")]
    InvalidArguments,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Divide by zero")]
    DivideByZero,
    #[msg("Overflow")]
    Overflow,
    #[msg("Underflow")]
    Underflow,
    #[msg("Insufficient supply")]
    InsufficientSupply,
}
