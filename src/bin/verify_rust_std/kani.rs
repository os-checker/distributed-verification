use crate::Result;
use eyre::Context;
use std::{env, process::Command};

/// Env var `STD_LIBRARY=path/to/library` must be set.
//
// Kani generates `kani-list.json` if succeeds.
pub fn list(args: &[String]) -> Result<()> {
    let kani = KaniArgs::new_for_list(args);
    if is_debug() {
        kani.debug();
    }
    if kani.exec()? {
        println!("kani-list.json is done.");
        Ok(())
    } else {
        bail!("kani-list.json failed to be generated.")
    }
}

/// Env var `STD_LIBRARY=path/to/library` must be set.
///
/// The arguments are harness names that will be passed with
/// `--include-pattern` and `--harness` options to minimize run time.
//
// Run kani verification.
pub fn run(v_harness: &[String]) -> Result<()> {
    let kani = KaniArgs::new_for_run(v_harness);
    if is_debug() {
        kani.debug();
    }
    if kani.exec()? {
        println!("Kani verification is done.");
        Ok(())
    } else {
        bail!("Kani verification is failed.")
    }
}

#[derive(Default)]
struct KaniArgs {
    args: Vec<String>,
}

impl KaniArgs {
    fn new_for_run(v_harness: &[String]) -> Self {
        let mut this = Self::basic();
        this.args.push("--output-format=terse".to_owned());
        this.args.push("-j".to_owned());

        // See https://github.com/model-checking/kani/issues/4079#issuecomment-3459290399
        this.args.reserve(v_harness.len() * 4);
        for arg in v_harness
            .iter()
            .flat_map(|h| ["--include-pattern", h.as_str(), "--harness", h.as_str()])
        {
            this.args.push(arg.to_owned());
        }

        this
    }

    fn new_for_list(args: &[String]) -> Self {
        let mut this = Self::basic();

        this.add_slice(&["--list", "--format=json"]);
        this.add_slice(LIST_ARGS);
        this.add_slice(args);
        this
    }

    fn basic() -> Self {
        let mut this = Self::default();
        this.args.push("autoharness".to_owned());
        this.add_slice(&["--std".to_owned(), std_library().unwrap()]);
        this.add_slice(UNSTABLE_ARGS);
        this
    }

    fn add_slice<T: Clone + Into<String>>(&mut self, v: &[T]) {
        self.args.extend(v.iter().map(|s| s.clone().into()));
    }

    fn exec(self) -> Result<bool> {
        Ok(Command::new("kani").args(self.args).spawn()?.wait()?.success())
    }

    fn debug(&self) {
        let mut v = vec!["kani"];
        v.extend(self.args.iter().map(|arg| arg.as_str()));
        println!("cmd=`{}`", v.join(" "));
    }
}

fn is_debug() -> bool {
    env::var("DEBUG").is_ok_and(|s| !matches!(&*s.to_lowercase(), "0" | "false"))
}

fn std_library() -> Result<String> {
    env::var("STD_LIBRARY")
        .with_context(|| "Env var `STD_LIBRARY` must be set to the library path in verify-rut-std.")
}

const UNSTABLE_ARGS: &[&str] = &[
    "-Zautoharness",
    "-Zfunction-contracts",
    "-Zmem-predicates",
    "-Zfloat-lib",
    "-Zc-ffi",
    "-Zloop-contracts",
    "-Zquantifiers",
    "-Zstubbing",
    "-Zunstable-options",
    "--harness-timeout=10m",
    "--default-unwind=1000",
];

const LIST_ARGS: &[&str] = &[
    "--include-pattern",
    "<(.+)[[:space:]]as[[:space:]](.+)>::disjoint_bitor",
    "--include-pattern",
    "<(.+)[[:space:]]as[[:space:]](.+)>::unchecked_disjoint_bitor",
    "--include-pattern",
    "<(.+)[[:space:]]as[[:space:]]iter::range::Step>::backward_unchecked",
    "--include-pattern",
    "<(.+)[[:space:]]as[[:space:]]iter::range::Step>::forward_unchecked",
    "--include-pattern",
    "alloc::__default_lib_allocator::",
    "--include-pattern",
    "alloc::layout::Layout::from_size_align",
    "--include-pattern",
    "ascii::ascii_char::AsciiChar::from_u8",
    "--include-pattern",
    "char::convert::from_u32_unchecked",
    "--include-pattern",
    "core_arch::x86::__m128d::as_f64x2",
    "--include-pattern",
    "convert::num::<impl.convert::From<num::nonzero::NonZero<",
    "--include-pattern",
    "num::<impl.i8>::unchecked_add",
    "--include-pattern",
    "num::<impl.i16>::unchecked_add",
    "--include-pattern",
    "num::<impl.i32>::unchecked_add",
    "--include-pattern",
    "num::<impl.i64>::unchecked_add",
    "--include-pattern",
    "num::<impl.i128>::unchecked_add",
    "--include-pattern",
    "num::<impl.isize>::unchecked_add",
    "--include-pattern",
    "num::<impl.u8>::unchecked_add",
    "--include-pattern",
    "num::<impl.u16>::unchecked_add",
    "--include-pattern",
    "num::<impl.u32>::unchecked_add",
    "--include-pattern",
    "num::<impl.u64>::unchecked_add",
    "--include-pattern",
    "num::<impl.u128>::unchecked_add",
    "--include-pattern",
    "num::<impl.usize>::unchecked_add",
    "--include-pattern",
    "num::<impl.i8>::unchecked_neg",
    "--include-pattern",
    "num::<impl.i16>::unchecked_neg",
    "--include-pattern",
    "num::<impl.i32>::unchecked_neg",
    "--include-pattern",
    "num::<impl.i64>::unchecked_neg",
    "--include-pattern",
    "num::<impl.i128>::unchecked_neg",
    "--include-pattern",
    "num::<impl.isize>::unchecked_neg",
    "--include-pattern",
    "num::<impl.i8>::unchecked_sh",
    "--include-pattern",
    "num::<impl.i16>::unchecked_sh",
    "--include-pattern",
    "num::<impl.i32>::unchecked_sh",
    "--include-pattern",
    "num::<impl.i64>::unchecked_sh",
    "--include-pattern",
    "num::<impl.i128>::unchecked_sh",
    "--include-pattern",
    "num::<impl.isize>::unchecked_sh",
    "--include-pattern",
    "num::<impl.u8>::unchecked_sh",
    "--include-pattern",
    "num::<impl.u16>::unchecked_sh",
    "--include-pattern",
    "num::<impl.u32>::unchecked_sh",
    "--include-pattern",
    "num::<impl.u64>::unchecked_sh",
    "--include-pattern",
    "num::<impl.u128>::unchecked_sh",
    "--include-pattern",
    "num::<impl.usize>::unchecked_sh",
    "--include-pattern",
    "num::<impl.i8>::unchecked_sub",
    "--include-pattern",
    "num::<impl.i16>::unchecked_sub",
    "--include-pattern",
    "num::<impl.i32>::unchecked_sub",
    "--include-pattern",
    "num::<impl.i64>::unchecked_sub",
    "--include-pattern",
    "num::<impl.i128>::unchecked_sub",
    "--include-pattern",
    "num::<impl.isize>::unchecked_sub",
    "--include-pattern",
    "num::<impl.u8>::unchecked_sub",
    "--include-pattern",
    "num::<impl.u16>::unchecked_sub",
    "--include-pattern",
    "num::<impl.u32>::unchecked_sub",
    "--include-pattern",
    "num::<impl.u64>::unchecked_sub",
    "--include-pattern",
    "num::<impl.u128>::unchecked_sub",
    "--include-pattern",
    "num::<impl.usize>::unchecked_sub",
    "--include-pattern",
    "num::<impl.i8>::wrapping_sh",
    "--include-pattern",
    "num::<impl.i16>::wrapping_sh",
    "--include-pattern",
    "num::<impl.i32>::wrapping_sh",
    "--include-pattern",
    "num::<impl.i64>::wrapping_sh",
    "--include-pattern",
    "num::<impl.i128>::wrapping_sh",
    "--include-pattern",
    "num::<impl.isize>::wrapping_sh",
    "--include-pattern",
    "num::<impl.u8>::wrapping_sh",
    "--include-pattern",
    "num::<impl.u16>::wrapping_sh",
    "--include-pattern",
    "num::<impl.u32>::wrapping_sh",
    "--include-pattern",
    "num::<impl.u64>::wrapping_sh",
    "--include-pattern",
    "num::<impl.u128>::wrapping_sh",
    "--include-pattern",
    "num::<impl.usize>::wrapping_sh",
    "--include-pattern",
    "num::nonzero::NonZero::<i8>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<i16>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<i32>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<i64>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<i128>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<isize>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<u8>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<u16>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<u32>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<u64>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<u128>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<usize>::count_ones",
    "--include-pattern",
    "num::nonzero::NonZero::<i8>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<i16>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<i32>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<i64>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<i128>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<isize>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<u8>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<u16>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<u32>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<u64>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<u128>::rotate_",
    "--include-pattern",
    "num::nonzero::NonZero::<usize>::rotate_",
    "--include-pattern",
    "ptr::align_offset::mod_inv",
    "--include-pattern",
    "ptr::alignment::Alignment::as_nonzero",
    "--include-pattern",
    "ptr::alignment::Alignment::as_usize",
    "--include-pattern",
    "ptr::alignment::Alignment::log2",
    "--include-pattern",
    "ptr::alignment::Alignment::mask",
    "--include-pattern",
    "ptr::alignment::Alignment::new",
    "--include-pattern",
    "ptr::alignment::Alignment::new_unchecked",
    "--include-pattern",
    "time::Duration::from_micros",
    "--include-pattern",
    "time::Duration::from_millis",
    "--include-pattern",
    "time::Duration::from_nanos",
    "--exclude-pattern",
    "time::Duration::from_nanos_u128",
    "--include-pattern",
    "time::Duration::from_secs",
    "--exclude-pattern",
    "time::Duration::from_secs_f",
    "--include-pattern",
    "unicode::unicode_data::conversions::to_",
    "--exclude-pattern",
    "::precondition_check",
];
