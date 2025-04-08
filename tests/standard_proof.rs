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

    #[kani::proof]
    fn recursive_callees() {
        crate::top_level();
    }
}

pub fn top_level() {
    m::func1();
}

mod m {
    pub fn func1() {}
}
