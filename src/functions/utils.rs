use rustc_middle::ty::TyCtxt;
use rustc_smir::rustc_internal::internal;
use rustc_span::{Span, source_map::SourceMap};
use rustc_stable_hash::{StableHasher, hashers::SipHasher128};
use serde::Serialize;
use stable_mir::mir::mono::Instance;
use std::hash::Hasher;

/// Source code and potential source code before expansion.
///
/// The field order matters, since this struct implements Ord.
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct SourceCode {
    /// Function name.
    pub name: String,

    /// String of [`InstanceKind`].
    ///
    /// [`InstanceKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/stable_mir/mir/mono/enum.InstanceKind.html
    pub kind: String,

    // A file path where src lies.
    // The path is stripped with pwd or sysroot prefix.
    pub file: String,

    /// Source that a stable_mir span points to.
    pub src: String,

    /// Is the stable_mir span from a macro expansion?
    /// If it is from an expansion, what's the source code before expansion?
    /// * Some(_) happens when the src (stable_mir) span comes from expansion, and tells
    ///   the source before the expansion.
    /// * None if the src is not from a macro expansion.
    ///
    /// Refer to [#31] to know sepecific cases.
    ///
    /// [#31]: https://github.com/os-checker/distributed-verification/issues/31
    pub before_expansion: Option<String>,
}

impl SourceCode {
    pub fn with_hasher(&self, hasher: &mut StableHasher<SipHasher128>) {
        hasher.write_str(&self.name);
        hasher.write_str(&self.kind);
        hasher.write_str(&self.file);
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

/// Source code for a stable_mir span.
pub fn source_code_with(
    inst: &Instance,
    stable_mir_span: stable_mir::ty::Span,
    tcx: TyCtxt,
    src_map: &SourceMap,
    path_prefixes: [&str; 2],
) -> SourceCode {
    let span = internal(tcx, stable_mir_span);
    let src = span_to_source(span, src_map);
    let before_expansion = span.from_expansion().then(|| {
        let ancestor_span = span.find_oldest_ancestor_in_same_ctxt();
        span_to_source(ancestor_span, src_map)
    });

    let mut file = stable_mir_span.get_filename();
    for prefix in path_prefixes {
        if let Some(file_stripped) = file.strip_prefix(prefix) {
            file = file_stripped.to_owned();
            break;
        }
    }

    let name = inst.name();
    let kind = format!("{:?}", inst.kind);
    SourceCode { name, kind, file, src, before_expansion }
}
