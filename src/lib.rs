use crypto_bigint::Uint;
use tracing::debug;

pub fn shr<const LIMBS: usize>(val: &Uint<LIMBS>, shift: u32) -> Uint<LIMBS> {
    assert!(shift < Uint::<LIMBS>::BITS, "Shift out of bounds");
    if cfg!(target_arch = "aarch64") {
        unsafe {
            return shr_aarch64(val, shift);
        }
    }
    debug!("Fallback to Rust implementation");
    val.shr(shift)
}

#[cfg(target_arch = "aarch64")]
unsafe fn shr_aarch64<const LIMBS: usize>(val: &Uint<LIMBS>, shift: u32) -> Uint<LIMBS> {
    let mut res = Uint::ZERO;
    let limbs = val.as_limbs();
    let out_limbs = res.as_limbs_mut();
    debug!("Asm shr: {:#X} >> {}", val, shift);
    unsafe {
        core::arch::asm!(
        "mov x6, #0",           // Init carry

        // Loop over the limbs
        "1:",
        "ldr x7, [x0], #8",     // x7 ← Memory[x0] (load 64-bit limb)
                                // x0 ← x0 + 8 (increment input pointer)
        "mov x8, x7",           // x8 ← x7 (preserve original limb value in x8)
        "lsr x7, x7, x3",       // Rights shift x7 by x3 steps and store in x7
        "orr x7, x7, x6",       // Combine with carry from previous limb, x7 ← x7 | x6
        "str x7, [x1], #8",     // Store shifted limb in the out_limbs (pointed to by x1)
                                // increment x1 by 8 bytes so it points to the next limb.
        "neg x9, x3",           // x9 ← -x3 (negate x3 to get the shift amount, which works
                                // because on ARM, negative shifts are mod 64, so neg
                                // works out to be `64 - x3`)
        "lsl x6, x8, x9",       // Left shift the original limb (x8) by 64 - x3 (x9 steps).
        "subs x2, x2, #1",      // x2 ← x2 - 1 (decrement limb counter)
                                // Sets condition flags (Z=1 when x2 reaches 0)
        "b.ne 1b",              // Branch to label 1 if Z=0 (Not Equal)


        // =============================================
        // Register Operand Bindings
        // =============================================
        // Input pointer to source limbs (read-only, auto-incremented in loop)
        in("x0") limbs.as_ptr(),

        // Output pointer for result limbs (auto-incremented in loop)
        inout("x1") out_limbs.as_mut_ptr() => _,

        // Limb counter (decremented in loop)
        inout("x2") LIMBS => _,

        // Shift amount (constant during operation)
        in("x3") shift,
        // Carry register (explicitly initialized, must be declared)
        out("x6") _,

        // =============================================
        // Register Preservation
        // =============================================
        // Declares all caller-saved registers as clobbered
        clobber_abi("C")
        );
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto_bigint::Uint;
    use test_log::test;
    use tracing::info;

    #[test]
    fn test_shr() {
        let num = Uint::<4>::from(0x123456789ABCDEF0u128);
        let shifted = shr(&num, 4);
        info!("\nShifted:\t\t {:X}\nOriginal:\t\t{:X}", shifted, num);
        assert_eq!(shifted, Uint::<4>::from(0x0123456789ABCDEF0u128 >> 4));
    }

    #[test]
    fn test_shr_zero() {
        const L: usize = 4;
        let num = Uint::<L>::from(0u128);
        let shifted = shr(&num, 4);

        assert_eq!(shifted, Uint::<L>::ZERO);
    }
}
