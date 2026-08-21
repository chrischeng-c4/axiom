# strings — external contract (as-is, 2026-07-15)

Domain map: `tech-design/strings/ARCHITECTURE.md` (EC surface section). Verdict law: `HARNESS.md`.
Oracle = live python3.12 byte-diff; xfail = acknowledged gap, still contract (skip, never executed).

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

Core-owned dirs (live-counted 2026-07-15, `tests/cpython/` relative), grouped per ARCHITECTURE's EC-surface split.

**Methods + formatting** (str/bytes surface, `%`/`.format`/f-string, `repr`/`str`):

| Dir | .py | xfail | Notes |
|---|---|---|---|
| `behavior/std-libs/string` | 75 | 15 | `Formatter`/`Template` classes, str case/find/replace/split/strip/zfill |
| `behavior/std-libs/strings` | 11 | 11 | core string-object surface (100% xfail) |
| `behavior/std-libs/userstring` | 3 | 3 | `UserString` (100% xfail) |
| `behavior/std-libs/encode_basestring_ascii` | 1 | 1 | json ascii-escape helper (100% xfail) |
| `behavior/core/fstring` | 85 | 65 | f-string value/spec grammar (76% xfail) |
| `_regression/core/fstring` | 13 | 0 | f-string regression probes |
| `_regression/core/string_escapes` | 6 | 0 | escape-sequence regressions |
| **subtotal** | **194** | **95** | |

**Unicode + surrogates** (codepoint storage, PEP 3131/263, `unicodedata`):

| Dir | .py | xfail | Notes |
|---|---|---|---|
| `behavior/std-libs/unicode` | 151 | 136 | CPython `test_unicode` port (90% xfail) |
| `behavior/std-libs/unicode_identifiers` | 3 | 2 | PEP 3131 identifier normalization |
| `behavior/std-libs/unicodedata` | 49 | 29 | category/combining/numeric/decimal + `name()`/`lookup()` |
| `behavior/std-libs/stringprep` | 1 | 1 | RFC 3454 tables (100% xfail) |
| `behavior/std-libs/source_encoding` | 44 | 43 | PEP 263 source-encoding declarations (98% xfail) |
| **subtotal** | **248** | **211** | |

**Codecs** (encode/decode families, error handlers, `codecs` module):

| Dir | .py | xfail | Notes |
|---|---|---|---|
| `behavior/std-libs/codecs` | 307 | 220 | core encode/decode surface (72% xfail) |
| `behavior/std-libs/codeccallbacks` | 40 | 37 | error-handler callback protocol (92.5% xfail) |
| `behavior/std-libs/multibytecodec` | 36 | 35 | CJK multi-byte codec machinery (97% xfail) |
| `behavior/std-libs/charmapcodec` | 4 | 4 | charmap codec (100% xfail) |
| `behavior/std-libs/codecencodings_{cn,hk,iso2022,jp,kr,tw}` | 7 | 7 | 6 dirs, 1 fixture each except iso2022 (2), all xfail |
| `behavior/std-libs/codecmaps_{cn,hk,jp,kr,tw}` | 5 | 5 | 5 dirs, 1 fixture each, all xfail |
| `behavior/std-libs/asian_codecs` | 3 | 3 | 100% xfail |
| `behavior/std-libs/_encoded_words` | 39 | 39 | RFC 2047 email word encoding (100% xfail) |
| **subtotal** | **441** | **350** | |

**Struct** (`struct.pack`/`unpack` byte-string formatting, ARCHITECTURE-assigned to this domain):

| Dir | .py | xfail | Notes |
|---|---|---|---|
| `behavior/std-libs/struct` | 60 | 19 | core pack/unpack |
| `behavior/std-libs/struct_fields` | 11 | 11 | named-field structs (100% xfail) |
| `behavior/std-libs/structures` | 36 | 36 | ctypes-adjacent structure layouts (100% xfail) |
| `behavior/std-libs/unaligned_structures` | 2 | 2 | 100% xfail |
| **subtotal** | **109** | **68** | |

**Per-dimension counterparts** for the 4 modules that also carry `errors/`/`real_world/`/`surface/` twins (README.md's `behavior/std-libs/<mod>, errors/std-libs/<mod>, real_world/std-libs/<mod>` rule); all live, 0 xfail:

| Module | `errors/std-libs` .py | `real_world/std-libs` .py | `surface/std-libs` .py |
|---|---|---|---|
| string | 9 | 1 | 25 |
| codecs | 14 | 1 | 63 |
| unicodedata | 9 | 1 | 32 |
| struct | 12 | 1 | 17 |
| stringprep | — | — | 27 |
| **subtotal** | **44** | **4** | **164** |

**Totals: 1,204 fixtures, 724 xfail (480 live).** Core-owned dirs alone: 992 fixtures, 724 xfail (268 live);
per-dimension counterparts add 212 fixtures, all live. Runtime rejects that belong to this domain (e.g. `LookupError`
on unknown/non-text codec names, `UnicodeEncodeError`/`UnicodeDecodeError` construction) are proven POSITIVELY inside
these fixtures — not walls.

## Negative contract — what must be REJECTED

No wall dimension of its own (README.md domain map — `type/` is the one negative dimension and it belongs to
type-system). Walls that sit over this domain's module surface, owned by type-system per the dimension rule
(live-counted 2026-07-15):

| `type/std-libs/` dir | .py | xfail |
|---|---|---|
| `string` | 14 | 7 |
| `string_templatelib` | 4 | 3 |
| `stringprep` | 19 | 0 |
| `unicodedata` | 33 | 0 |
| `codecs` | 63 | 0 |
| `_codecs` | 47 | 0 |
| `_multibytecodec` | 13 | 0 |
| `_struct` | 11 | 0 |
| `encodings` + `encodings_*` (97 dirs, cp*/iso8859*/mac_*/utf_*/...) | 427 | 0 |
| **total** | **631** | **10** |

A compile reject in any dir from the Positive contract above is a type-system false positive by definition
(README.md dimension rule) — not a strings-domain fault.

## Known contract gaps

- **Surrogate sidecar is leak-by-design and thread-local**: `cleanup_all_surrogate_strings` is an intentional no-op
  (`string_ops.rs:79`); correctness depends on a freed pointer address never being reused by a later non-surrogate
  string, and surrogate strings never cross threads. No fixture can exercise this over process life. Plain
  knowledge — the one related tracker, #92 ("str/bytes surrogate representation"), is CLOSED (`gh issue view 92`:
  audited 2026-07-13, "repro 已過時" / the original repro is stale) and does not cover this framing.
- **Sidecar guard set is narrow — zero-fixture live-divergence surface**: only the `is*` family, `ord`, `len`,
  `encode`, and equality consult `SURROGATE_STRINGS`; `mb_str_getitem`/slice/`upper`/`find` etc. silently operate on
  the single-char PUA sentinel for a surrogate-backed string instead of raising or resolving the real codepoints.
  Neither `unicode/` nor `unicode_identifiers/` isolates this beyond the guarded ops. #1770 does not cover this
  (its real scope is unrelated object-model/numeric divergences, no mention of strings/unicode/surrogates); new
  finding, no existing tracker found.
- **`unicodedata.name()` placeholder is invisible to the current corpus**: it returns `"UNICODE CHAR {:04X}"`
  (`unicodedata_mod.rs:302`), not the real Unicode name. Spot-checked the only 2 live fixtures that touch `.name()`
  — `name_lookup_round_trip.py` (asserts mamba's own `name()`↔`lookup()` round-trip) and
  `name_default_for_unnamed_char.py` (NUL falls into the narrow control-char "no name" branch) — both byte-match the
  oracle without ever comparing a real name string, so the placeholder never surfaces. #1770 does not cover this
  (its real scope is unrelated object-model/numeric divergences); new finding, no existing tracker found.
- **`codecs.register_error`/`lookup_error`/`register`/`open` are literal stubs** (`codecs_mod.rs:800-818`,
  return `None`); a user `register_error` call succeeds silently but the handler is never consulted — encode/decode
  only dispatch the hardcoded built-in names (`xmlcharrefreplace_errors`/`backslashreplace_errors`/
  `namereplace_errors` module fns are stubs too, `:1582-1590`). Spot-checked `codeccallbacks/`: 17 of the 19
  fixtures that call `register_error` are xfail; the 2 live ones (`test_encode_odd_bytes_replacement`,
  `test_longstrings`) don't exercise the registry path. Not actually untracked: open issue #238 (codecs
  error-handler registry epic, per `module-hazards.md`) already documents exactly this fact; tracked: #238
  (the dir's 92.5% xfail rate is itself part of the separate #1768 un-xfail campaign).
- **Codec allow-list is broader than the implementation**: `KNOWN_CODECS` (`string_ops.rs:1772-2144`, ~373 entries
  incl. `1252`, `936`, `big5`, iso8859 variants) accepts many names that `mb_str_encode_with`/`mb_bytes_decode_with` never
  actually implement; an unimplemented-but-known name silently falls back to lenient UTF-8 decode/encode
  (`bytes_ops.rs:774`) instead of the real transform or an error — wrong bytes, no signal. The CJK/multi-byte probe
  surface that would catch this is effectively 100% xfail today (`codecencodings_*`/`codecmaps_*`: 11/11 dirs;
  `asian_codecs`: 3/3; `multibytecodec`: 35/36). New finding, no existing tracker.
- **Mass-xfail strata hide most of the corpus**: `behavior/core/fstring` 65/85 (76%), `behavior/std-libs/codecs`
  220/307 (72%), `unicode` 136/151 (90%), `source_encoding` 43/44 (98%), `_encoded_words` 39/39 (100%),
  `struct_fields`/`structures`/`unaligned_structures` (100% each) — xfail is full skip, so hidden passes are
  expected at these rates per the un-xfail campaign's measured stale-rate precedent; tracked: #1768.

## Verification

```bash
# inner loop (seconds; paths relative to tests/cpython) — from apps/mamba/tests/harness/cpython/
python3 tools/sweep.py behavior/std-libs/string behavior/std-libs/strings behavior/std-libs/userstring \
  behavior/std-libs/encode_basestring_ascii behavior/core/fstring _regression/core/fstring \
  _regression/core/string_escapes                                    # methods + formatting slice
python3 tools/sweep.py behavior/std-libs/unicode behavior/std-libs/unicode_identifiers \
  behavior/std-libs/unicodedata behavior/std-libs/stringprep behavior/std-libs/source_encoding  # unicode + surrogates slice
python3 tools/sweep.py behavior/std-libs/codecs behavior/std-libs/codeccallbacks behavior/std-libs/multibytecodec \
  behavior/std-libs/charmapcodec behavior/std-libs/codecencodings_cn behavior/std-libs/codecencodings_hk \
  behavior/std-libs/codecencodings_iso2022 behavior/std-libs/codecencodings_jp behavior/std-libs/codecencodings_kr \
  behavior/std-libs/codecencodings_tw behavior/std-libs/codecmaps_cn behavior/std-libs/codecmaps_hk \
  behavior/std-libs/codecmaps_jp behavior/std-libs/codecmaps_kr behavior/std-libs/codecmaps_tw \
  behavior/std-libs/asian_codecs behavior/std-libs/_encoded_words     # codecs slice
python3 tools/sweep.py behavior/std-libs/struct behavior/std-libs/struct_fields behavior/std-libs/structures \
  behavior/std-libs/unaligned_structures                             # struct slice
# cargo gate slice (datatest filter is a path substring; one dir per filter run)
cargo test -p mamba --release --test conformance -- behavior/std-libs/unicode
cargo test -p mamba --release --test conformance -- behavior/std-libs/codecs
# C2 slices: tests/harness/cpython/config/perf/pins/{string_concat_1382,unicodedata_1261}.toml
cargo test -p mamba --release --test perf_pin -- perf_pin
# manifests: config/manifests/std-libs/{string,codecs,struct,unicodedata}.toml exist (generator-fed);
# strings/userstring/fstring/multibytecodec/etc. are hand-authored/pre-manifest. After fixture edits run
# tools/fixture_lint.py.
# full C1 gate (~3 min; this domain's slice rides inside it) — never concurrent with a cargo build
cargo test -p mamba --release --test conformance
```
