use crate::{BoxStr, ProofKind, SerFunction};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    hash::Hash,
    path::{Component, MAIN_SEPARATOR, Path, PathBuf},
};

// ************ `kani list --json` ************

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct KaniListHarnesses {
    pub inner: IndexMap<BoxStr, IndexSet<BoxStr>>,
}

impl KaniListHarnesses {
    pub fn normalize_file_path(&mut self) {
        pub fn normalize_path(path: &Path) -> String {
            let mut components = path.components().peekable();
            let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
                components.next();
                PathBuf::from(c.as_os_str())
            } else {
                PathBuf::new()
            };

            for component in components {
                match component {
                    Component::Prefix(..) => unreachable!(),
                    Component::RootDir => {
                        ret.push(component.as_os_str());
                    }
                    Component::CurDir => {}
                    Component::ParentDir => {
                        ret.pop();
                    }
                    Component::Normal(c) => {
                        ret.push(c);
                    }
                }
            }
            ret.display().to_string()
        }

        let mut inner = IndexMap::with_capacity(self.inner.len());
        for (path, value) in self.inner.iter_mut() {
            let key = normalize_path(&PathBuf::from(&**path)).into();
            inner.insert(key, std::mem::take(value));
        }
        self.inner = inner;
    }

    pub fn strip_path_prefix(&mut self, prefix: &str) {
        let mut map = IndexMap::with_capacity(self.inner.len());
        for (key, val) in self.inner.iter_mut() {
            let val = std::mem::take(val);
            match key.strip_prefix(prefix) {
                Some(stripped) => _ = map.insert(stripped.into(), val),
                None => {
                    // Some files refer to registry folder:
                    // `/home/runner/.cargo/registry/src/innerdex.crates.io-1949cf8
                    // c6b5b557f/addr2line-0.25.0/src/line.rs`
                    // So we should keep it as it is.
                    warn!("The key `{key}`\nis not stripped with prefix `{prefix}`.");
                    _ = map.insert(key.clone(), val);
                }
            }
        }
        map.sort_unstable_keys();
        self.inner = map;
    }

    pub fn strip_path_closure_name(&mut self, v_text: &[&str]) {
        for set in self.inner.values_mut() {
            let mut stripped = IndexSet::with_capacity(set.len());
            for name in set.iter() {
                let mut new_name = String::new();
                for text in v_text {
                    new_name = name.replace(text, "");
                }
                stripped.insert(new_name.into());
            }
            *set = stripped;
        }
    }

    fn files(&self) -> Vec<&str> {
        self.inner.keys().map(|s| &**s).collect()
    }

    /// filter is a closure where the first argument is filename, and the second is function name.
    /// If filter returns true, the function name will be appended to v.
    fn names<'harness>(
        &'harness self,
        v: &mut Vec<&'harness str>,
        mut filter: impl FnMut(&str, &str) -> bool,
    ) {
        for (file, harnesses) in &self.inner {
            for name in harnesses {
                if filter(file, name) {
                    v.push(&**name);
                }
            }
        }
    }

    fn file_func_name(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().flat_map(|(k, v)| v.iter().map(|func| (&**k, &**func)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct ContractedFunction {
    /// The fully qualified name the user gave to the function (i.e. includes the module path).
    pub function: BoxStr,
    /// The (currently full-) path to the file this function was declared within.
    pub file: BoxStr,
    /// The pretty names of the proof harnesses (`#[kani::proof_for_contract]`) for this function
    pub harnesses: Box<[BoxStr]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub struct Totals {
    pub standard_harnesses: usize,
    pub contract_harnesses: usize,
    pub functions_under_contract: usize,
}

/// The datastructure generated from `kani list --json`.
///
/// ref: https://github.com/model-checking/kani/blob/b64e59de669cd77b625cc8c0b9a94f29117a0ff7/kani-driver/src/list/output.rs#L113
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct KaniListJson {
    pub kani_version: BoxStr,
    pub file_version: BoxStr,
    pub standard_harnesses: KaniListHarnesses,
    pub contract_harnesses: KaniListHarnesses,
    pub contracts: IndexSet<ContractedFunction>,
    pub totals: Totals,
}

impl KaniListJson {
    /// Call this immediately after deserialization is done, especially before strip_path_prefix.
    pub fn normalize_file_path(&mut self) {
        self.standard_harnesses.normalize_file_path();
        self.contract_harnesses.normalize_file_path();
    }

    // FIXME: merge this and the raw one.
    pub fn strip_path_prefix<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        let path = path
            .canonicalize()
            .unwrap_or_else(|err| panic!("Unable to canonicalize {path:?}:\n{err}"));
        let prefix = &format!("{}{MAIN_SEPARATOR}", path.to_str().unwrap());

        self.standard_harnesses.strip_path_prefix(prefix);
        self.contract_harnesses.strip_path_prefix(prefix);
    }

    /// This function is used in tests.
    pub fn strip_path_prefix_raw(&mut self, prefix: &str) {
        self.standard_harnesses.strip_path_prefix(prefix);
        self.contract_harnesses.strip_path_prefix(prefix);
    }

    pub fn strip_path_closure_name(&mut self, text: &[&str]) {
        self.standard_harnesses.strip_path_closure_name(text);
        self.contract_harnesses.strip_path_closure_name(text);
    }

    pub fn files(&self) -> serde_json::Value {
        serde_json::json!({
            "standard_harnesses": self.standard_harnesses.files(),
            "contract_harnesses": self.contract_harnesses.files()
        })
    }

    pub fn harness_names(&self, mut filter: impl FnMut(&str, &str) -> bool) -> IndexSet<&str> {
        let totals = &self.totals;
        let len = totals.standard_harnesses + totals.contract_harnesses;
        let mut v = Vec::with_capacity(len);

        self.standard_harnesses.names(&mut v, &mut filter);
        self.contract_harnesses.names(&mut v, &mut filter);

        let duplicates = count_gt1(&v);
        assert!(duplicates.is_empty(), "Function name duplicates: {duplicates:#?}");
        vec_to_set(&v)
    }

    pub fn file_func_name(&self) -> impl Iterator<Item = (&str, &str)> {
        self.standard_harnesses.file_func_name().chain(self.contract_harnesses.file_func_name())
    }
}

fn count_gt1<T: Copy + Hash + Eq>(v: &[T]) -> Vec<(T, u32)> {
    let mut map = IndexMap::with_capacity(v.len());
    for key in v {
        map.entry(*key).and_modify(|n| *n += 1).or_insert(1u32);
    }
    map.into_iter().filter(|(_, n)| *n != 1).collect()
}

fn vec_to_set<T: Copy + Hash + Eq + Ord>(v: &[T]) -> IndexSet<T> {
    let mut set: IndexSet<_> = v.iter().copied().collect();
    set.sort_unstable();
    set
}

// ************ difference ************

pub struct MergedHarnesses<'a> {
    pub functions: &'a [SerFunction],
    pub standard: Box<[&'a SerFunction]>,
    pub contract: Box<[&'a SerFunction]>,
}

impl MergedHarnesses<'_> {
    pub fn new(functions: &[SerFunction]) -> MergedHarnesses<'_> {
        let cap = functions.len();
        let mut standard = Vec::with_capacity(cap);
        let mut contract = Vec::with_capacity(cap);
        for func in functions {
            match func.proof_kind {
                Some(ProofKind::Standard) => standard.push(func),
                Some(ProofKind::Contract) => contract.push(func),
                None => (),
            }
        }
        MergedHarnesses {
            functions,
            standard: standard.into_boxed_slice(),
            contract: contract.into_boxed_slice(),
        }
    }

    pub fn function_names(&self, mut filter: impl FnMut(&SerFunction) -> bool) -> IndexSet<&str> {
        let v: Vec<_> = self.functions.iter().filter(|f| filter(f)).map(|f| &*f.name).collect();
        // FIXME: it's possible the function name will duplicate, need to figure out why.
        vec_to_set(&v)
    }
}

/// Merge hash json and kani-list.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MergeHashKaniList {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "crate")]
    pub krate: Option<Box<str>>,
    pub file: Box<str>,
    pub func: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub hash: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub proof_kind: Option<ProofKind>,
}

/// Compare two `MergeHashKaniList`, and returns the ones from new that don't match the old,
/// usually the ones don't have hash values or whoes hash values changed.
///
/// If new is sorted, especially directly from the stdout of verify_rust_std merge subcommand,
/// the result is sorted.
pub fn diff(old: &[MergeHashKaniList], new: &[MergeHashKaniList]) -> Vec<MergeHashKaniList> {
    let set: HashSet<_> = old.iter().collect();
    new.iter().filter(|item| item.hash.is_none() || !set.contains(item)).cloned().collect()
}
