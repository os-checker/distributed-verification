#[cfg(kani)]
mod verify {
    #[kani::requires(a>0)]
    fn f(a: u8) {}

    #[kani::proof]
    fn recursive_callees() {
        f(0);
    }
}
