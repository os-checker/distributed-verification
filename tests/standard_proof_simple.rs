#[cfg(kani)]
mod verify {

    #[kani::proof]
    fn recursive_callees() {
        crate::top_level();
    }
}

pub fn top_level() {
    m::func1();
}

mod m {
    pub fn func1() {
        let _a = "".trim();
    }
}
