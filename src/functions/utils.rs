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

    /// Mangled function name.
    pub mangled_name: String,

    /// String of [`InstanceKind`].
    ///
    /// [`InstanceKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/stable_mir/mir/mono/enum.InstanceKind.html
    pub kind: String,

    // A file path where src lies.
    // The path is stripped with pwd or sysroot prefix.
    pub file: String,

    /// Source that a stable_mir span points to.
    pub src: String,

    /// The count of macro backtraces.
    pub macro_backtrace_len: usize,

    /// Is the stable_mir span from a macro expansion?
    /// If it is from an expansion, what's the source code before expansion?
    /// * Some(_) happens when the src (stable_mir) span comes from expansion, and tells
    ///   the source before the expansion.
    /// * None if the src is not from a macro expansion.
    ///
    /// Refer to [#31] to know sepecific cases.
    ///
    /// [#31]: https://github.com/os-checker/distributed-verification/issues/31
    pub macro_backtrace: Vec<MacroBacktrace>,
}

impl SourceCode {
    pub fn with_hasher(&self, hasher: &mut StableHasher<SipHasher128>) {
        hasher.write_str(&self.name);
        hasher.write_str(&self.mangled_name);
        hasher.write_str(&self.kind);
        hasher.write_str(&self.file);
        hasher.write_str(&self.src);
        hasher.write_length_prefix(self.macro_backtrace_len);
        for m in &self.macro_backtrace {
            hasher.write_str(&m.callsite);
            hasher.write_str(&m.defsite);
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct MacroBacktrace {
    callsite: String,
    defsite: String,
}

fn span_to_source(span: Span, src_map: &SourceMap) -> String {
    src_map
        .span_to_source(span, |text, x, y| {
            let src = &text[x..y];
            // debug!("[{x}:{y}]\n{src}");
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

    if span.from_expansion() {
        debug!(
            "[find_oldest_ancestor_in_same_ctxt] {}",
            span_to_source(span.find_oldest_ancestor_in_same_ctxt(), src_map)
        );
        debug!("[source_callsite] {}", span_to_source(span.source_callsite(), src_map));
        if let Some(parent_callsite) = span.parent_callsite() {
            debug!("[parent_callsite] {}", span_to_source(parent_callsite, src_map));
        }
        for m in span.macro_backtrace() {
            debug!("[macro_backtrace - callsite] {}", span_to_source(m.call_site, src_map));
            debug!("[macro_backtrace - defsite ] {}", span_to_source(m.def_site, src_map));
        }
        debug!("\n");
    }
    let macro_backtrace: Vec<_> = span
        .macro_backtrace()
        .map(|m| MacroBacktrace {
            callsite: span_to_source(m.call_site, src_map),
            defsite: span_to_source(m.def_site, src_map),
        })
        .collect();
    let macro_backtrace_len = macro_backtrace.len();

    let mut file = stable_mir_span.get_filename();
    for prefix in path_prefixes {
        if let Some(file_stripped) = file.strip_prefix(prefix) {
            file = file_stripped.to_owned();
            break;
        }
    }

    let name = inst.name();
    let mangled_name = inst.mangled_name();
    let kind = format!("{:?}", inst.kind);
    SourceCode { name, mangled_name, kind, file, src, macro_backtrace_len, macro_backtrace }
}
