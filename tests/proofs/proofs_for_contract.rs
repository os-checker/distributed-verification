#[cfg(kani)]
mod verify {
    #[kani::requires(a > 0)]
    fn contract_requires(a: u8) {}

    #[kani::proof_for_contract(contract_requires)]
    fn single_contract_requires() {
        contract_requires(0);
    }

    #[kani::ensures(|&ret| ret > 0)]
    fn contract_ensures(a: u8) -> u8 {
        a
    }

    #[kani::proof_for_contract(contract_ensures)]
    fn single_with_contract_ensures() {
        contract_ensures(0);
    }

    #[kani::requires(a > 0)]
    #[kani::ensures(|&ret| ret > 0)]
    fn contract(a: u8) -> u8 {
        a
    }

    #[kani::proof_for_contract(contract)]
    fn single_contract() {
        let val = contract(1);
        assert!(val > 0);
    }
}
