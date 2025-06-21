use distributed_verification::{InstKind, MacroBacktrace, ProofKind, SourceCode};
use rustc_middle::ty::TyCtxt;
use rustc_smir::rustc_internal::internal;
use rustc_span::{Span, source_map::SourceMap};
use stable_mir::{
    CrateDef,
    mir::mono::{Instance, InstanceKind},
};

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
    stable_mir_span: stable_mir::ty::Span,
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

    SourceCode {
        name: inst.name(),
        inst_kind: new_inst_kind(inst.kind),
        proof_kind,
        file,
        attrs,
        src,
        macro_backtrace_len,
        macro_backtrace,
    }
}

fn get_all_attrs(tcx: TyCtxt, inst: &Instance) -> (Vec<String>, Option<ProofKind>) {
    use super::kani::{PROOF, PROOF_FOR_CONTRACT};
    use rustc_attr_data_structures::AttributeKind;
    use rustc_hir::Attribute;

    let def_id = internal(tcx, inst.def.def_id());
    let mut proof_kind = None;
    let attrs = tcx
        .get_all_attrs(def_id)
        .filter(|attr| match attr {
            Attribute::Unparsed(unparsed) => {
                let idents = &unparsed.path.segments;
                if let Some(first) = idents.first()
                    && first.as_str() == super::TOOL
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
            Attribute::Parsed(AttributeKind::Repr(_)) => true,
            // FIXME: add support for #[align] when the toolchain bumps over 2025-06-19
            //
            // * https://github.com/rust-lang/rust/commit/1fdf2b562070ec98c5b32ee67b8c6d8145127a6e
            // * https://github.com/rust-lang/rfcs/pull/3806
            // Attribute::Parsed(AttributeKind::Align(_)) => true,
            _ => false,
        })
        .map(|attr| rustc_hir_pretty::attribute_to_string(&tcx, attr))
        .collect();
    (attrs, proof_kind)
}

pub fn vec_convertion<U, T: From<U>>(vec: Vec<U>) -> Vec<T> {
    vec.into_iter().map(T::from).collect()
}
