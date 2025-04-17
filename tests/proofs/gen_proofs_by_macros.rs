macro_rules! gen {
    ($name:ident, $block:block) => {
        #[kani::proof]
        fn $name() $block
    }
}

#[cfg(kani)]
mod verify {
    gen! { proof1, { assert_eq!(kani::any::<u8>(), 0) }}
    gen! { proof2, { assert_eq!(kani::any::<u8>(), 0) }}
    gen! { proof3, { assert_eq!(kani::any::<u8>(), 1) }}
}
