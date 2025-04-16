use rustc_middle::ty::TyCtxt;
use rustc_smir::rustc_internal::internal;
use rustc_span::{Span, source_map::SourceMap};
use rustc_stable_hash::{StableHasher, hashers::SipHasher128};
use serde::Serialize;
use stable_mir::{CrateDef, mir::mono::Instance};
use std::{cmp::Ordering, hash::Hasher};

/// Source code and potential source code before expansion.
///
/// Refer to
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct SourceCode {
    // TODO:
    // file: String,
    //
    /// Source that a stable_mir span points to.
    pub src: String,
    /// Is the stable_mir span from a macro expansion?
    /// If it is from an expansion, what's the source code before expansion?
    /// * Some(_) happens when the src (stable_mir) span comes from expansion, and tells
    ///   the source before the expansion.
    /// * None if the src is not from a macro expansion.
    pub before_expansion: Option<String>,
}

impl SourceCode {
    pub fn with_hasher(&self, hasher: &mut StableHasher<SipHasher128>) {
        hasher.write_str(&self.src);
        match &self.before_expansion {
            Some(text) => {
                hasher.write_u8(1);
                hasher.write_str(text);
            }
            None => hasher.write_u8(0),
        }
    }
}

fn span_to_source(span: Span, src_map: &SourceMap) -> String {
    src_map
        .span_to_source(span, |text, x, y| {
            let src = &text[x..y];
            debug!("[{x}:{y}]\n{src}");
            Ok(src.to_owned())
        })
        .unwrap()
}

/// Source code for a span.
fn source_code(span: Span, src_map: &SourceMap) -> SourceCode {
    let src = span_to_source(span, src_map);
    let before_expansion = span.from_expansion().then(|| {
        let ancestor_span = span.find_oldest_ancestor_in_same_ctxt();
        span_to_source(ancestor_span, src_map)
    });
    SourceCode { src, before_expansion }
}

/// Source code for a stable_mir span.
pub fn source_code_with(
    stable_mir_span: stable_mir::ty::Span,
    tcx: TyCtxt,
    src_map: &SourceMap,
) -> SourceCode {
    let span = internal(tcx, stable_mir_span);
    source_code(span, src_map)
}

// FIXME: takes body and returns SourceCode
pub fn source_code_of_body(
    inst: &Instance,
    tcx: TyCtxt,
    src_map: &SourceMap,
) -> Option<SourceCode> {
    inst.body().map(|body| source_code_with(body.span, tcx, src_map))
}

pub fn cmp_callees(a: &Instance, b: &Instance, tcx: TyCtxt, src_map: &SourceMap) -> Ordering {
    let filename_a = file_path(a);
    let filename_b = file_path(b);
    match filename_a.cmp(&filename_b) {
        Ordering::Equal => (),
        ord => return ord,
    }

    let body_a = source_code_of_body(a, tcx, src_map);
    let body_b = source_code_of_body(b, tcx, src_map);
    body_a.cmp(&body_b)
}

pub fn file_path(inst: &Instance) -> String {
    use std::sync::LazyLock;
    static PREFIXES: LazyLock<[String; 2]> = LazyLock::new(|| {
        let mut pwd = std::env::current_dir().unwrap().into_os_string().into_string().unwrap();
        pwd.push('/');

        let out = std::process::Command::new("rustc").arg("--print=sysroot").output().unwrap();
        let sysroot = std::str::from_utf8(&out.stdout).unwrap().trim();
        let sysroot = format!("{sysroot}/lib/rustlib/src/rust/");
        [pwd, sysroot]
    });

    let file = inst.def.span().get_filename();
    for prefix in &*PREFIXES {
        if let Some(file) = file.strip_prefix(prefix) {
            return file.to_owned();
        }
    }
    file
}
