#[cfg(kani)]
mod verify {
    #[kani::requires(a > 0)]
    fn contract_requires(a: u8) {}

    #[kani::proof]
    fn standard_proof_with_contract_requires() {
        contract_requires(0);
    }

    #[kani::ensures(|&ret| ret > 0)]
    fn contract_ensures(a: u8) -> u8 {
        a
    }

    #[kani::proof]
    fn standard_proof_with_contract_ensures() {
        contract_ensures(0);
    }
}
