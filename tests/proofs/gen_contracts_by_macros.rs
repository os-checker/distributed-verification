macro_rules! gen {
    ($proof:ident: $contract_name:ident($arg:ident), $e:expr) => {
        #[kani::requires($e)]
        fn $contract_name($arg: u8) -> u8 { $arg }

        #[kani::proof_for_contract($contract_name)]
        fn $proof() {
            $contract_name(kani::any::<u8>());
        }
    }
}

#[cfg(kani)]
mod verify {
    gen! { proof1: contract1(arg), arg > 0}
    gen! { proof2: contract2(arg), arg != 0}
    gen! { proof3: contract3(arg), arg < 10}
}
