#[cfg(kani)]
mod adhoc {
    #[kani::proof]
    fn callee_defined_in_proof() {
        fn f() -> u8 {
            1
        }
        assert!(f() == 1);
    }

    #[kani::proof]
    fn closure_in_proof() {
        let f = || 1;
        assert!(f() == 1);
    }

    fn proof_in_fn_item() {
        #[kani::proof]
        fn proof() {
            assert!(kani::any::<u8>() == 1);
        }
    }
}
