use distributed_verification::{InstKind, MacroBacktrace, ProofKind, SourceCode};
use itertools::Itertools;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_public::{
    CrateDef,
    mir::mono::{Instance, InstanceKind},
    rustc_internal::internal,
};
use rustc_span::{Span, source_map::SourceMap};
use rustc_stable_hash::{
    FromStableHash, StableHasher,
    hashers::{SipHasher128, SipHasher128Hash},
};
use std::hash::Hash;

fn new_inst_kind(kind: InstanceKind) -> Option<InstKind> {
    Some(match kind {
        InstanceKind::Item => return None,
        InstanceKind::Intrinsic => InstKind::Intrinsic,
        InstanceKind::Virtual { .. } => InstKind::Virtual,
        InstanceKind::Shim => InstKind::Shim,
    })
}

fn span_to_snippet(span: Span, src_map: &SourceMap) -> String {
    src_map.span_to_snippet(span).unwrap()
}

/// Source code for a stable_mir span.
pub fn source_code_with(
    inst: &Instance,
    stable_mir_span: rustc_public::ty::Span,
    tcx: TyCtxt,
    src_map: &SourceMap,
    path_prefixes: [&str; 2],
) -> SourceCode {
    let span = internal(tcx, stable_mir_span);
    let src = span_to_snippet(span, src_map);
    let (attrs, proof_kind) = get_all_attrs(tcx, inst);

    let macro_backtrace: Vec<_> = span
        .macro_backtrace()
        .map(|m| MacroBacktrace {
            callsite: span_to_snippet(m.call_site, src_map),
            defsite: span_to_snippet(m.def_site, src_map),
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

    let path = defid_to_path(internal(tcx, inst.def.def_id()), tcx);

    SourceCode {
        name: inst.name(),
        inst_kind: new_inst_kind(inst.kind),
        proof_kind,
        file,
        attrs,
        src,
        macro_backtrace_len,
        macro_backtrace,
        path,
    }
}

// FIXME: need to comfirm how `tcx.def_path_str(def_id)` differs from this
fn defid_to_path(did: DefId, tcx: TyCtxt) -> Box<str> {
    tcx.def_path_str(did).into()
    // use std::fmt::Write;
    //
    // let mut buf = String::with_capacity(64);
    // let def_path = tcx.def_path(did);
    // let fmt_path = def_path
    //     .data
    //     .iter()
    //     .map(|d| match d.data.name() {
    //         rustc_hir::definitions::DefPathDataName::Named(symbol) => symbol,
    //         rustc_hir::definitions::DefPathDataName::Anon { namespace } => namespace,
    //     })
    //     .format_with("::", |ele, f| f(&format_args!("{}", ele.as_str())));
    // if did.is_local() {
    //     let crate_name = tcx.crate_name(def_path.krate);
    //     let crate_name = crate_name.as_str();
    //     _ = write!(&mut buf, "{crate_name}::{fmt_path}");
    // } else {
    //     _ = write!(&mut buf, "{fmt_path}");
    // }
    //
    // buf.into()
}

fn get_all_attrs(tcx: TyCtxt, inst: &Instance) -> (Vec<String>, Option<ProofKind>) {
    use super::kani::{PROOF, PROOF_FOR_CONTRACT, TOOL};
    use rustc_hir::Attribute;
    use rustc_hir::attrs::AttributeKind;

    let def_id = internal(tcx, inst.def.def_id());
    let mut proof_kind = None;
    let attrs = tcx
        .get_all_attrs(def_id)
        .iter()
        .filter(|attr| match attr {
            Attribute::Unparsed(unparsed) => {
                let idents = &unparsed.path.segments;
                if let Some(first) = idents.first()
                    && first.as_str() == TOOL
                {
                    if let Some(second) = idents.get(1) {
                        match second.as_str() {
                            PROOF => proof_kind = Some(ProofKind::Standard),
                            PROOF_FOR_CONTRACT => proof_kind = Some(ProofKind::Contract),
                            _ => (),
                        }
                    }

                    return true;
                }
                false
            }
            Attribute::Parsed(AttributeKind::Repr { .. }) => true,
            Attribute::Parsed(AttributeKind::Align { .. }) => true,
            // * https://github.com/rust-lang/rust/commit/1fdf2b562070ec98c5b32ee67b8c6d8145127a6e
            // * https://github.com/rust-lang/rfcs/pull/3806
            // Attribute::Parsed(AttributeKind::Align(_)) => true,
            _ => false,
        })
        .map(|attr| rustc_hir_pretty::attribute_to_string(&tcx, attr))
        .collect();
    (attrs, proof_kind)
}

// ************* hash *************
struct Hash128(Box<str>);

impl FromStableHash for Hash128 {
    type Hash = SipHasher128Hash;

    fn from(SipHasher128Hash([a, b]): SipHasher128Hash) -> Hash128 {
        Hash128(format!("{a}{b}").into())
    }
}

pub fn stable_hash<T: Hash>(val: T) -> Box<str> {
    let mut hasher = StableHasher::<SipHasher128>::new();
    val.hash(&mut hasher);
    let Hash128(hash) = hasher.finish();
    hash
}

pub struct StreamHasher(StableHasher<SipHasher128>);

impl StreamHasher {
    pub fn new() -> Self {
        StreamHasher(StableHasher::<SipHasher128>::new())
    }

    pub fn append<T: Hash>(&mut self, val: T) {
        val.hash(&mut self.0);
    }

    pub fn finish(self) -> Box<str> {
        let Hash128(hash) = self.0.finish();
        hash
    }
}
// ************* hash *************
