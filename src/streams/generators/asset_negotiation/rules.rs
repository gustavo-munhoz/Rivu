use super::AssetRule;
use super::domain::{AMOUNT, COLOR, DELAY, PAYMENT, PRICE, idx};
use rand::Rng;

/// Evaluates the target concept for a given rule and a 5-tuple of
/// domain indices (color, price, payment, amount, delay).
///
/// Returns 0 for "interested" and 1 for "notInterested".
///
/// This function is pure (no RNG) and is used by both the generator and tests.
/// The five rules correspond to the published patterns:
/// - **R1:** `price=normal ∧ amount=high` ∨ `color=brown ∧ price=veryLow ∧ delay=high`
/// - **R2:** `price=high ∧ amount=veryHigh ∧ delay=high`
/// - **R3:** `price=veryLow ∧ payment=0 ∧ amount=high` ∨ `color=red ∧ price=low ∧ payment=30`
/// - **R4:** `color=black ∧ payment=90 ∧ delay=veryLow` ∨ `color=magenta ∧ price=high ∧ delay=veryLow`
/// - **R5:** `color=blue ∧ payment=60 ∧ amount=low ∧ delay=normal` ∨ `color=cyan ∧ amount=low ∧ delay=normal`
#[inline]
pub fn evaluate_rule_idx(rule: AssetRule, vals: &[usize; 5]) -> usize {
    let (c, p, pay, a, d) = (vals[0], vals[1], vals[2], vals[3], vals[4]);

    let (
        c_black,
        c_brown,
        c_red,
        c_cyan,
        c_magenta,
        p_norm,
        p_vlow,
        p_low,
        p_high,
        a_low,
        a_high,
        a_vhigh,
        pay_0,
        pay_30,
        pay_60,
        pay_90,
        d_norm,
        d_high,
        d_vlow,
    ) = (
        idx(&COLOR, "black"),
        idx(&COLOR, "brown"),
        idx(&COLOR, "red"),
        idx(&COLOR, "cyan"),
        idx(&COLOR, "magenta"),
        idx(&PRICE, "normal"),
        idx(&PRICE, "veryLow"),
        idx(&PRICE, "low"),
        idx(&PRICE, "high"),
        idx(&AMOUNT, "low"),
        idx(&AMOUNT, "high"),
        idx(&AMOUNT, "veryHigh"),
        idx(&PAYMENT, "0"),
        idx(&PAYMENT, "30"),
        idx(&PAYMENT, "60"),
        idx(&PAYMENT, "90"),
        idx(&DELAY, "normal"),
        idx(&DELAY, "high"),
        idx(&DELAY, "veryLow"),
    );

    let interested = match rule {
        AssetRule::R1 => {
            (p == p_norm && a == a_high) || (c == c_brown && p == p_vlow && d == d_high)
        }
        AssetRule::R2 => p == p_high && a == a_vhigh && d == d_high,
        AssetRule::R3 => {
            (p == p_vlow && pay == pay_0 && a == a_high)
                || (c == c_red && p == p_low && pay == pay_30)
        }
        AssetRule::R4 => {
            (c == c_black && pay == pay_90 && d == d_vlow)
                || (c == c_magenta && p == p_high && d == d_vlow)
        }
        AssetRule::R5 => {
            (c == c_cyan && a == a_low && d == d_norm)
                || (/* variante blue */a == a_low && d == d_norm && pay == pay_60 && c == idx(&COLOR, "blue"))
        }
    };

    if interested { 0 } else { 1 }
}

/// Mutates vals **in place** to satisfy the **positive class** (class 0)
/// for the given rule. Used when balance_classes = true to construct a
/// valid class-0 example in case the random sample produced class 0 but the
/// alternation requires class 1 (the label is then set to 1 after mutation).
///
/// The exact patterns match the ones documented in [evaluate_rule_idx].
pub fn make_true_sample_idx<R: Rng + ?Sized>(rule: AssetRule, rng: &mut R, vals: &mut [usize; 5]) {
    let (
        c_black,
        c_brown,
        c_red,
        c_cyan,
        c_blue,
        c_magenta,
        p_norm,
        p_vlow,
        p_low,
        p_high,
        a_low,
        a_high,
        a_vhigh,
        pay_0,
        pay_30,
        pay_60,
        pay_90,
        d_norm,
        d_vlow,
        d_high,
    ) = (
        idx(&COLOR, "black"),
        idx(&COLOR, "brown"),
        idx(&COLOR, "red"),
        idx(&COLOR, "cyan"),
        idx(&COLOR, "blue"),
        idx(&COLOR, "magenta"),
        idx(&PRICE, "normal"),
        idx(&PRICE, "veryLow"),
        idx(&PRICE, "low"),
        idx(&PRICE, "high"),
        idx(&AMOUNT, "low"),
        idx(&AMOUNT, "high"),
        idx(&AMOUNT, "veryHigh"),
        idx(&PAYMENT, "0"),
        idx(&PAYMENT, "30"),
        idx(&PAYMENT, "60"),
        idx(&PAYMENT, "90"),
        idx(&DELAY, "normal"),
        idx(&DELAY, "veryLow"),
        idx(&DELAY, "high"),
    );

    match rule {
        AssetRule::R1 => {
            if rng.random::<bool>() {
                vals[1] = p_norm;
                vals[3] = a_high;
            } else {
                vals[0] = c_brown;
                vals[1] = p_vlow;
                vals[4] = d_high;
            }
        }
        AssetRule::R2 => {
            vals[1] = p_high;
            vals[3] = a_vhigh;
            vals[4] = d_high;
        }
        AssetRule::R3 => {
            if rng.random::<bool>() {
                vals[1] = p_vlow;
                vals[2] = pay_0;
                vals[3] = a_high;
            } else {
                vals[0] = c_red;
                vals[1] = p_low;
                vals[2] = pay_30;
            }
        }
        AssetRule::R4 => {
            if rng.random::<bool>() {
                vals[0] = c_black;
                vals[2] = pay_90;
                vals[4] = d_vlow;
            } else {
                vals[0] = c_magenta;
                vals[1] = p_high;
                vals[4] = d_vlow;
            }
        }
        AssetRule::R5 => {
            if rng.random::<bool>() {
                vals[0] = c_blue;
                vals[2] = pay_60;
                vals[3] = a_low;
                vals[4] = d_norm;
            } else {
                vals[0] = c_cyan;
                vals[3] = a_low;
                vals[4] = d_norm;
            }
        }
    }
}
