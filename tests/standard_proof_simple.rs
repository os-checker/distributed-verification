#[cfg(kani)]
mod verify {
    #[kani::requires(a>0)]
    fn f(a: u8) {}

    #[kani::proof]
    fn proof() {
        f(0);
    }
}
