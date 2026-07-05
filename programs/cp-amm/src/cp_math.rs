use anchor_lang::prelude::*;

use crate::AMMError;

pub const BASIS_POINTS: u64 = 10_000;

/// a * b / c
#[inline(always)]
fn mul_div(a: u64, b: u64, c: u64) -> Result<u64> {
    require!(c != 0, AMMError::DivideByZero);
    let result = (a as u128)
        .checked_mul(b as u128)
        .ok_or(AMMError::Overflow)?
        / (c as u128);
    let result = u64::try_from(result).map_err(|_| AMMError::Overflow)?;
    Ok(result)
}

/// sqrt(val)
#[inline(always)]
fn sqrt(val: u128) -> u128 {
    if val < 2 {
        return val;
    }
    let mut x = val;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + val / x) / 2;
    }
    x
}

#[inline(always)]
fn initial_add_liquidity(amount_a: u64, amount_b: u64) -> Result<u64> {
    let product = (amount_a as u128)
        .checked_mul(amount_b as u128)
        .ok_or(AMMError::Overflow)?;
    let value = u64::try_from(sqrt(product)).map_err(|_| AMMError::Overflow)?;
    Ok(value)
}

#[inline(always)]
fn lp_amount_for_add_liquidity(
    amount_a: u64,
    amount_b: u64,
    reserve_a: u64,
    reserve_b: u64,
    lp_supply: u64,
) -> Result<u64> {
    let from_a = mul_div(amount_a, lp_supply, reserve_a)?;
    let from_b = mul_div(amount_b, lp_supply, reserve_b)?;
    Ok(from_a.min(from_b))
}

#[inline(always)]
pub fn add_liquidity_amount(
    amount_a_desired: u64,
    amount_b_desired: u64,
    reserve_a: u64,
    reserve_b: u64,
    lp_supply: u64,
) -> Result<(u64, u64, u64)> {
    if reserve_a == 0 || reserve_b == 0 || lp_supply == 0 {
        return Ok((
            amount_a_desired,
            amount_b_desired,
            initial_add_liquidity(amount_a_desired, amount_b_desired)?,
        ));
    }
    let amount_b_optimal = mul_div(amount_a_desired, reserve_b, reserve_a)?;
    if amount_b_optimal <= amount_b_desired {
        Ok((
            amount_a_desired,
            amount_b_optimal,
            lp_amount_for_add_liquidity(
                amount_a_desired,
                amount_b_optimal,
                reserve_a,
                reserve_b,
                lp_supply,
            )?,
        ))
    } else {
        let amount_a_optimal = mul_div(amount_b_desired, reserve_a, reserve_b)?;
        Ok((
            amount_a_optimal,
            amount_b_desired,
            lp_amount_for_add_liquidity(
                amount_a_optimal,
                amount_b_desired,
                reserve_a,
                reserve_b,
                lp_supply,
            )?,
        ))
    }
}

#[inline(always)]
pub fn withdraw_liquidity_amount(
    lp_ammount: u64,
    reserve_a: u64,
    reserve_b: u64,
    lp_supply: u64,
) -> Result<(u64, u64)> {
    require!(
        lp_supply >= lp_ammount && lp_ammount > 0,
        AMMError::InsufficientSupply
    );
    let amount_a = mul_div(lp_ammount, reserve_a, lp_supply)?;
    let amount_b = mul_div(lp_ammount, reserve_b, lp_supply)?;
    Ok((amount_a, amount_b))
}

#[inline(always)]
pub fn swap_amount(amount_in: u64, reserve_in: u64, reserve_out: u64, fee_bps: u16) -> Result<u64> {
    require!(
        reserve_in > 0 && reserve_out > 0,
        AMMError::InsufficientSupply
    );
    let in_after_fee = mul_div(amount_in, BASIS_POINTS - (fee_bps as u64), BASIS_POINTS)?;
    let amount_out = (in_after_fee as u128)
        .checked_mul(reserve_out as u128)
        .ok_or(AMMError::Overflow)?
        / (in_after_fee as u128)
            .checked_add(reserve_in as u128)
            .ok_or(AMMError::Overflow)?;
    let value = u64::try_from(amount_out).map_err(|_| AMMError::Overflow)?;
    Ok(value)
}
