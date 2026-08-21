# stdlib — external contract (as-is, 2026-07-15)

Domain map: `tech-design/stdlib/ARCHITECTURE.md` (EC surface section) + `tech-design/stdlib/module-hazards.md`
(shared traps checklist, referenced not restated). Verdict law: `HARNESS.md`.
Oracle = live python3.12 byte-diff; xfail = acknowledged gap, still contract (skip, never executed).

Scope note: this domain's EC surface (`behavior|errors|real_world|surface/std-libs/<mod>`) spans the WHOLE
std-libs fixture dimension — 31,538 fixtures / 21,141 xfail live today across `behavior` (24,718), `errors`
(915), `real_world` (511), `surface` (5,394) over ~580 per-module dirs. Modules owned by a sibling domain
(numbers/strings/collections/object-model/exceptions/...) keep their per-module counts in that domain's own
EC doc — this doc does not re-derive them. What follows is live-counted for the two mechanisms that belong to
*this* ARCHITECTURE.md specifically: the vendored `py_src` tree and the import-survival sentinel family.

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

**Vendored `py_src` modules** (9 total, `vendor_lib.rs:VENDORED_MODULES` — no native shell may exist for any
of these; source wins per the import-precedence invariant):

| Module | behavior .py/xfail | errors .py/xfail | real_world .py/xfail | surface .py/xfail | total .py/xfail |
|---|---|---|---|---|---|
| `colorsys` | 19/0 | 3/0 | 1/0 | 7/0 | 30/0 |
| `fileinput` | 56/47 | 0/0 | 0/0 | 12/0 | 68/47 |
| `getopt` | 14/7 | 5/0 | 1/0 | 8/0 | 28/7 |
| `getpass` | 14/14 | 0/0 | 0/0 | 3/0 | 17/14 |
| `mailbox` | 95/94 | 0/0 | 0/0 | 0/0 | 95/94 |
| `nturl2path` | 0/0 | 0/0 | 0/0 | 0/0 | 0/0 |
| `plistlib` | 57/48 | 0/0 | 0/0 | 0/0 | 57/48 |
| `quopri` | 11/2 | 0/0 | 0/0 | 0/0 | 11/2 |
| `uu` | 14/14 | 0/0 | 0/0 | 0/0 | 14/14 |
| **Total** | **280/226** | 8/0 | 2/0 | 30/0 | **320/226** |

**Import-survival sentinel family** (`module-hazards.md`; marker + readiness-exclusion tracked #1514/#1119):

| Family | Files | C1 fixtures (behavior+errors+real_world+surface) | C2 perf pins |
|---|---|---|---|
| Callable-sentinel shims (`"Minimal callable-dispatcher shim"` header, `#1496` pattern) | 53 | 0 for 49 pure-3rd-party shims; the other 4 (`bdb`/`compileall`/`concurrent_futures` #1261, `multiprocessing` #1476) grew real coverage under the same header — see gaps | 53/53 have one (e.g. `pydantic_core_1496.toml`, `grpclib_1514.toml`) |
| One-attr probe markers (`thirdparty_shells_mod.rs`, 104 names: click/rich/anyio/attrs/dateutil/...) | 104 | 0 | 0 |

Method: `grep -l "Minimal callable-dispatcher shim" src/runtime/stdlib/*.rs` → 53; cross-checked by `wc -l`
(49 files sit in the described 58–77 line band; the 4 outliers are 498–748 lines) and by `ls
tests/cpython/{behavior,errors,real_world,surface}/std-libs/<name>` per shim/marker name (none exist except
for the 4 outliers and `unittest_mock`, the real stdlib module distinct from the `mock` PyPI shim).

Note: the `#1496`/`#1261`/`#1476` numbers above are copied verbatim from the shim files' own doc-comment
headers, but none of the three resolves to a real GitHub issue on this repo (verified via `gh issue view`) —
they are stale/placeholder tracker references baked into the source comments themselves, not just this doc;
flagging here rather than inventing a replacement number.

## Negative contract — what must be REJECTED

None owned here. Per the README.md dimension rule, every `type/` wall — including ones that gate this
domain's own modules — belongs to type-system. For context only (not this domain's contract):

- The 9 vendored modules carry 95 curated `type/std-libs/<mod>` walls, 0 xfail (all enforced): colorsys 6,
  fileinput 6, getopt 3, getpass 1, mailbox 64, nturl2path 2, plistlib 7, quopri 4, uu 2. `getopt`'s
  `args`-shape wall and colorsys's `h: float` wall (`stdlib_sigs.rs`) are the curated-wall precedents the
  ARCHITECTURE.md extension-point row cites.
- Three sentinel-shim modules have wall dirs: `type/std-libs/typing_extensions` (37 walls, 27 xfail),
  `type/std-libs/bdb` (40 walls, 0 xfail), `type/std-libs/compileall` (3 walls, 0 xfail).
- All other sentinel/marker/vendored names have zero `type/` presence.

## Known contract gaps

- **Sentinel-shim family has zero C1 proof, only a C2 pin measuring a lie**: all 53 callable-sentinel shims
  have a perf pin but 0 fixtures in any run dimension for the 49 pure-3rd-party ones (pydantic/pydantic_core,
  boto3/botocore/s3transfer, the azure_*/google_*/googleapis_common_protos trio, grpcio/grpclib, redis,
  sqlalchemy, requests/httpx/aiohttp/aiofiles, flask/fastapi/starlette/uvicorn/gunicorn/werkzeug/wsgiref,
  psycopg, cryptography/pyopenssl, msgpack/orjson/jsonschema/marshmallow, attrs, hypothesis/mock/pytest/
  pytest_asyncio, urllib3, charset_normalizer/idna, protobuf, celery/kombu, alembic, jmespath,
  typing_extensions). The 105 one-attr probe markers in `thirdparty_shells_mod.rs` have neither a fixture
  nor a pin. This directly contradicts the ARCHITECTURE.md EC-surface claim that `surface/std-libs/<mod>`
  presence probes are "the only dimension sentinel shims genuinely satisfy" — live count shows even that
  dimension is empty for this family; the perf-pin C2 surface is the only signal that exists, and
  module-hazards.md is explicit it "measures a lie." Stale EC-surface doc claim, same shape as
  object-model.md's/import-system.md's path-drift findings; tracked: #1771.
- **`nturl2path` — the one module vendored specifically to prove the loader resolves a module with NO native
  shell at all (#867 AC2) — has zero fixtures in any dimension.** The architectural proof point it exists to
  demonstrate is currently untested. Not xfail-rot (there is nothing to rot); a plain coverage void. No
  existing tracker fits precisely — new finding, not filed.
- **Vendoring's positive contract is xfail-heavy for half the vendored set**: `mailbox` 94/95 (99%), `uu`
  14/14 (100%), `getpass` 14/14 (100%), `plistlib` 48/57 (84%), `fileinput` 47/56 (84%) xfail — the
  "vendoring flip is safe" claim for these 5 modules rests on very few (`mailbox`: 1; `uu`: 0) live
  byte-match fixtures, even though `vendor_lib.rs`'s own chronicle treats them as landed/stable. Same shape
  as the un-xfail campaign's measured stale-rate finding elsewhere in the corpus; tracked: #1768.
- **4 sentinel-shim files outgrew their own doc-comment**: `bdb_mod.rs`, `compileall_mod.rs`,
  `concurrent_futures_mod.rs` (#1261 each) and `multiprocessing_mod.rs` (#1476) still open with the
  "Minimal callable-dispatcher shim ... ships the Gate 2 module-attr-read perf surface" header verbatim at
  498–748 lines, long past the 58–77 line shape the header describes, and do carry real `behavior`/`surface`
  fixture coverage (unlike the other 49). Whether the #1514/#1119 readiness-exclusion logic still correctly
  treats these 4 as "shim, exclude" or has started counting stale-labeled-but-real coverage is unaudited here
  — no tracker found for this specific staleness; new finding, not filed.
- **DictKey hash-domain / runtime-key aliasing / active-module-stack / TESTFN / vendoring-flip hazards**: see
  `module-hazards.md`'s shared-traps table and `ARCHITECTURE.md`'s Known hazards — both already own tracked
  refs per-hazard (xml_etree #1629, tempfile/os.PathLike #1630, codecs error-handler registry #238's epic);
  not re-derived here.

## Verification

```bash
# inner loop (seconds; runner-parity verdicts; set MAMBA_BIN after a release build) — from
# apps/mamba/tests/harness/cpython/ — vendored py_src slice (only dirs that exist; nturl2path has none)
python3 tools/sweep.py \
  behavior/std-libs/{colorsys,fileinput,getopt,getpass,mailbox,plistlib,quopri,uu} \
  errors/std-libs/{colorsys,getopt} real_world/std-libs/{colorsys,getopt} \
  surface/std-libs/{colorsys,fileinput,getopt,getpass}
# cargo gate slice (datatest filter is a path substring; one dir per filter run)
cargo test -p mamba --release --test conformance -- std-libs/colorsys
cargo test -p mamba --release --test conformance -- std-libs/mailbox
# C2 slice for the sentinel-shim family (measures the shim, not behavior — see gaps above)
cargo test -p mamba --release --test perf_pin -- perf_pin
# full C1 gate (~3 min; this domain's slice — and every sibling domain's — rides inside it)
cargo test -p mamba --release --test conformance
```

Manifests: 4 of the 9 vendored modules have one (`config/manifests/std-libs/{colorsys,getopt}.toml`,
`config/manifests/std-libs/cpython312_surface/{fileinput,getpass}.toml`) — `mailbox`/`nturl2path`/`plistlib`/
`quopri`/`uu` remain hand-authored/pre-manifest. All 4 sentinel-shim "outlier" modules also have one
(`config/manifests/std-libs/{bdb,compileall,concurrent_futures}.toml` +
`config/manifests/std-libs/cpython312_surface/concurrent_futures.toml`,
`config/manifests/std-libs/multiprocessing.toml` +
`config/manifests/std-libs/cpython312_surface/multiprocessing.toml`); the 49 pure-3rd-party shims and the
one-attr probe family have none. Gate readings are the only progress signal; per-fix evidence = before/after
readings on the issue (README.md rule).
