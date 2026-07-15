# strings — architecture (as-is, 2026-07-15)

The str/bytes/bytearray text domain: unicode storage, the str + bytes method
surfaces, all formatting (`%`, `.format`, f-string), and codec encode/decode +
`unicodedata`/`codecs`/`encodings` shims. getattr→method routing that *reaches*
these lives in object-model (`../object-model/`); the spec engine here is
shared by numeric `format()` but numeric coercion is `../numbers/`.

## Responsibilities

- Unicode text storage + the lone-surrogate sidecar (Rust `String` can't hold
  surrogates natively).
- str surface (`dispatch_str_method`) + bytes/bytearray surface
  (`dispatch_bytes_method`); comparison, hashing, slicing, `join`/`split`.
- Formatting: `%`, `str.format`/`format_map`, f-string values, the
  `[[fill]align][sign][#][0][width][,][.prec][type]` spec mini-language, and
  `repr`/`str` of every value (`value_to_string`).
- Codec encode/decode for the built-in encoding families + built-in
  error-handler names; `codecs`/`unicodedata`/`encodings` module surfaces.

## Key structures & invariants

| Structure | file:symbol | Rule |
|---|---|---|
| str storage | `rc.rs:488 ObjData::Str(String)` | native **UTF-8 Rust `String`** — NOT CPython's flexible Latin-1/UCS-2/UCS-4 rep; length/index are codepoint-based |
| bytes | `rc.rs:497 Bytes(Vec<u8>)` | immortal-immutable value |
| bytearray | `rc.rs:498 ByteArray(MbRwLock<Vec<u8>>)` | mutable, interior `MbRwLock` |
| surrogate sidecar | `string_ops.rs:13-16` | strings containing U+D800..DFFF are stored as the PUA sentinel `"\u{e000}"`; real codepoints live in a `thread_local SURROGATE_STRINGS: HashMap<ptr_as_usize, Vec<u32>>` keyed by **object pointer address** |
| surrogate lookup | `string_ops.rs:85 surrogate_codepoints` | valid only if the object *is* the sentinel AND ptr key present; `surrogate_len`/`surrogate_single_codepoint`/`is_surrogate_backed_string` (`:1354`) are the guards sprinkled across `is*`/`ord`/`len` |
| hashing | `string_ops.rs:25 string_hash_value` | one process-wide `RandomState` (`:19`), `>>17`; surrogate strings hash via `codepoints_hash_value` (`:34`) so an equal surrogate/non-surrogate pair agrees |
| ErrorsKind | `bytes_ops.rs:809` | decode error mode: Strict/Ignore/Replace/SurrogateEscape/SurrogatePass |

Invariant: any str op that must be surrogate-correct calls the sidecar guards
FIRST; ops that don't (see hazards) silently operate on the 1-char sentinel.

## Control flow

1. `s.method(...)` → object-model getattr (`class/mod.rs:18567,20604,21076`)
   → `string_ops.rs:5382 dispatch_str_method(name, recv, args)` — a big
   name-string `match`; bytes/bytearray → `bytes_ops.rs:2202
   dispatch_bytes_method`.
2. `str(x)`/`repr(x)` → `builtins/str_conversion.rs:5` / `builtins/mod.rs:4008`
   → `string_ops.rs:4878 value_to_string` (containers via `repr_in_container`,
   floats via `python_float_repr:2689`, bytes via `format_bytes_inline:4823`).
3. `s.encode(enc,err)` → `string_ops.rs:2160 mb_str_encode_with`: lowercase
   name → hardcoded family arm (utf-8/-sig, utf-16/32 ±BE/LE/BOM, ascii,
   latin-1, idna, punycode…); surrogate-backed input branches to
   `encode_surrogate_codepoints_{utf8,ascii,utf16,utf32}` (`:393-533`); errors
   handled inline (ignore/replace/strict + surrogateescape/pass).
4. `b.decode(enc,err)` → `bytes_ops.rs:665 mb_bytes_decode_with` → `ErrorsKind`
   → `decode_utf8_value:890` / `decode_ascii_value:932` / `decode_latin1` /
   `decode_utf16` / `decode_utf32`; surrogate paths at `:827,:853`.
5. `codecs.encode/decode(...)` → `codecs_mod.rs:709/750` **delegate straight to**
   `string_ops::mb_str_encode_with` / `bytes_ops::mb_bytes_decode_with`
   (`:737,:777`); text-only escape codecs (unicode_escape, raw_unicode_escape,
   string_escape) have their own paths at `codecs_mod.rs:969-1230`.
6. Formatting: `%`→`mb_str_percent_format:3592` (tuple/dict/scalar arg flatten);
   `.format`→`mb_str_format:4510`/`_kwargs:4662`/`_map:4276` (field parse
   `split_format_field:3956`, nested spec `resolve_nested_spec:4024`);
   f-string→`mb_fstring_value:2668`+`mb_format_value:2629`; all converge on
   `apply_format_spec:2796` (the spec engine) and `format_complex_spec:3444`.

## CPython-parity semantics

| Behavior | mamba contract |
|---|---|
| str/bytes boundary | distinct `ObjKind`; `in`/concat/compare require same side. `bytes(str)` **needs** the checked path: `mb_bytes_new_checked` (`bytes_ops.rs:486`) raises `TypeError: string argument without an encoding`; raw `mb_bytes_new:372` silently UTF-8-encodes — lowering must pick the checked/encoded variant |
| indexing/len | codepoint (`char`) based, O(n) `chars().collect()` per index (`mb_str_getitem:594`); matches CPython semantics, not perf |
| UnicodeError fields | encode via `raise_unicode_encode_error_instance` (`string_ops.rs:283`) with CPython reason strings `ordinal not in range(128)`/`(256)`; decode via `raise_unicode_decode_error` (`bytes_ops.rs:77`). `str(e)` recomputes from fields — see `../exceptions/construction-and-rendering.md` (message-only raise renders empty) |
| error handlers | only the built-in names are honored (strict/ignore/replace/xmlcharrefreplace/backslashreplace/namereplace/surrogateescape/surrogatepass, `codecs_mod.rs:638`); an unknown name → `LookupError` |
| surrogateescape/pass | round-trips lone surrogates through the sidecar (`decode_utf8_surrogateescape_value:827`, encode `0xDC80..DCFF`→original byte `:405`) |
| normalization | real NFC/NFD/NFKC/NFKD via `unicode_normalization` crate (`unicodedata_mod.rs:399,607`) |
| `%` vs `.format` | independent hand-rolled parsers; auto/manual field-numbering mutual exclusion enforced (`mark_auto/manual_numbering:3934`) |

## Known hazards

- **Surrogate sidecar is pointer-keyed + never GC'd** (`string_ops.rs:79
  cleanup_all_surrogate_strings` is an intentional NO-OP) — entries leak for
  process life; correctness relies on freed-pointer addresses not being reused
  by a later non-surrogate string. thread_local ⇒ surrogate strings do not
  cross threads.
- **Most str ops ignore the sidecar** — `mb_str_getitem`/slice/`upper`/`find`
  etc. see only the 1-char sentinel for a surrogate-backed string; only the
  `is*` family, `ord`, `len`, `encode`, and equality consult it. Silent wrong
  results outside those. (memory index flags #92 lone-surrogate as next.)
- **`codecs.register_error`/`lookup_error`/`register`/`open` are NO-OP stubs**
  (`codecs_mod.rs:800-818`) returning `None`; a user-registered error handler
  succeeds silently but is **never invoked** — encode/decode only dispatch the
  hardcoded built-in names. `xmlcharrefreplace_errors`/`backslashreplace_errors`/
  `namereplace_errors` module fns are also stubs (`:1582-1590`).
- **`unicodedata.name()` is a placeholder** — returns `"UNICODE CHAR {:04X}"`
  (`unicodedata_mod.rs:302`), not the real Unicode name; `name_impl` models "no
  name" narrowly as control chars only. category/combining/numeric/decimal ARE
  real (`unicode_properties`).
- **Codec name coverage is an allow-list** — families are matched by lowercased
  literal; non-enumerated-but-known text codecs fall back to lenient UTF-8
  (`bytes_ops.rs:774`), genuine-unknown → `LookupError`, non-text codecs
  (base64/quopri) → the "not a text encoding" `LookupError`.
- **Format/spec engines are large hand-written match arms** (`apply_format_spec`
  ~520 lines) — easy to drift from CPython's grammar edge cases (sign+fill,
  `,`/`_` grouping, `n`/`g` types, complex).

## Extension points

- New str/bytes method: add a `match` arm in `dispatch_str_method`
  (`string_ops.rs:5382`) / `dispatch_bytes_method` (`bytes_ops.rs:2202`) — the
  method itself is a free `mb_str_*`/`mb_bytes_*` fn.
- New encoding family: add an arm in `mb_str_encode_with`/`mb_bytes_decode_with`
  AND register the canonical name in `is_known_codec`/`nontext_codec_name`
  (`string_ops.rs`) so decode fallback/LookupError stay correct.
- Real error-handler registry: replace the `codecs_mod.rs:800-812` stubs with a
  registry the encode/decode inner loops consult (currently they can't).
- New format type char: extend `apply_format_spec:2796`.
- `unicodedata` real names: replace the `:302` placeholder with a name table.

## EC surface

- methods + formatting: `behavior/std-libs/{string,strings,userstring,
  encode_basestring_ascii}`, `behavior/core/fstring`, `_regression/core/{fstring,
  string_escapes}`.
- unicode + surrogates: `behavior/std-libs/{unicode,unicode_identifiers,
  unicodedata,stringprep,source_encoding}`.
- codecs: `behavior/std-libs/{codecs,codeccallbacks,multibytecodec,charmapcodec,
  codecencodings_*,codecmaps_*,asian_codecs,_encoded_words}`; `struct`:
  `behavior/std-libs/{struct,struct_fields,*structures}`.

Dimension mechanics: `tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`.
