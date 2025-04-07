#[cfg(kani)]
mod verify {
    #[kani::proof]
    fn standard_proof() {
        let val: u8 = kani::any();
        assert_eq!(val, 1, "Not eq to 1.");
    }

    #[kani::proof]
    #[allow(unused_mut)]
    fn standard_proof_empty() {}
}
