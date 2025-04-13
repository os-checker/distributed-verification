#[cfg(kani)]
mod verify {
    #[kani::requires(a > 0)]
    pub fn contract(a: u8) {}

    #[kani::proof]
    pub fn f() {
        contract(0);
    }
}
