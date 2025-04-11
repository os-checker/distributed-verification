#[cfg(kani)]
mod verify {
    #[kani::requires(a > 0)]
    fn contract1(a: u8) {}

    #[kani::proof]
    fn standard_proof_with_contract_requires() {
        contract1(0);
    }
}
