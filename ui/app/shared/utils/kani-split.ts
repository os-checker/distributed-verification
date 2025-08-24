/** URL to split folder.
 * Construct`${URL_SPLIT_BASE}/${file}/${hash}.json` to fetch split DbFunction JSON.
 * */
export const URL_SPLIT_BASE = "https://raw.githubusercontent.com/os-checker/verify-rust-std_data/refs/heads/main/split";

export interface DbFunction {
  file: string,
  name: string,
  hash: string,
  hash_direct: string,
  proof_kind?: string,
  attrs?: string[],
  src: string,
  macro_backtrace?: string[],
  callees?: string[],
}

/** URL to hash.json which stores `{file, name, hash}` as query key map. */
export const URL_HASH_JSON = "https://raw.githubusercontent.com/os-checker/verify-rust-std_data/refs/heads/main/hash.json";

export interface HashJson {
  file: string,
  name: string,
  hash: string,
}

/** Find split JSON URL. Currently, function name as the search key. */
export function get_split_json(v_hash: HashJson[], name: string): string | null {
  const key = v_hash.find(h => h.name === name);
  if (key === undefined) return null;
  return `${URL_SPLIT_BASE}/${key.file}/${key.hash}.json`
}

/** Source code string including attributes. */
export function src(func: DbFunction): string {
  const attrs = func.attrs?.join("\n");

  if (!attrs) return func.src;
  return `${attrs}${func.src}`;
}
