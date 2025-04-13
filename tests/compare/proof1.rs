#[cfg(kani)]
mod verify {
    #[kani::proof]
    pub fn f() {
        let a = 1;
        assert_eq!(a + 1, 2);
    }
}
