macro_rules! outer {
    ($name:ident, $block:block) => {
        inner!($name, $block);
    };
}

macro_rules! inner {
    ($name:ident, $block:block) => {
        #[kani::proof]
        fn $name() $block
    }
}

#[cfg(kani)]
mod verify {
    outer! { proof1, { assert_eq!(kani::any::<u8>(), 0) }}
}
