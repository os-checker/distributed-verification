use rustc_middle::ty::TyCtxt;
use rustc_smir::rustc_internal::internal;
use rustc_span::{Span, source_map::SourceMap};
use stable_mir::{CrateDef, mir::mono::Instance};
use std::cmp::Ordering;

/// Source code for a span.
fn source_code(span: Span, src_map: &SourceMap) -> String {
    // println!("{span:?}\n{:?}\n\n", span.find_oldest_ancestor_in_same_ctxt());
    // _ = src_map.span_to_source(span, |text, x, y| {
    //     println!("(stable_mir span to internal span) [{span:?}]\n{}", &text[x..y]);
    //     Ok(())
    // });
    // let ancestor_span = span.find_oldest_ancestor_in_same_ctxt();
    // dbg!(span.from_expansion(), ancestor_span.from_expansion());
    // _ = src_map.span_to_source(ancestor_span, |text, x, y| {
    //     println!("(find_oldest_ancestor_in_same_ctxt) [{ancestor_span:?}]\n{}\n\n", &text[x..y]);
    //     Ok(())
    // });

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
    stable_mir_span: stable_mir::ty::Span,
    tcx: TyCtxt,
    src_map: &SourceMap,
) -> String {
    let span = internal(tcx, stable_mir_span);
    source_code(span, src_map)
}

pub fn source_code_of_body(inst: &Instance, tcx: TyCtxt, src_map: &SourceMap) -> Option<String> {
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
