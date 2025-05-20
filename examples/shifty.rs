use crypto_bigint::Uint;
use crypto_bigint_asm::shr;

fn main() {
    let num = Uint::<4>::from(0x123456789ABCDEF0u128);
    let shifted = shr(&num, 4);

    println!("Original:\t\t{:X}", num);
    println!("Shifted right by 4:\t{:X}", shifted);
}
