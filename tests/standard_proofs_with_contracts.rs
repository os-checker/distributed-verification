#[cfg(kani)]
mod verify {
    #[kani::requires(a > 0)]
    fn contract_requires(a: u8) {}

    #[kani::proof]
    fn standard_proof_with_contract_requires() {
        contract_requires(0);
    }
}
