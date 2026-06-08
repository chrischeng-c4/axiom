# urllib.parse — pure-string compute, wave-4 leader candidate

**Source:** `projects/mamba/vendor/typeshed/stdlib/urllib/parse.pyi` (354 lines).

## Surface (21-entry __all__)

- **Constants (7):** `uses_relative`, `uses_netloc`, `uses_params`, `non_hierarchical`,
  `uses_query`, `uses_fragment` (each `Final[list[str]]`), `scheme_chars` (`Final[str]`).
- **Def (15 forward fns; 12 + 3 overloads in __all__):** `urlparse`, `urlunparse`,
  `urljoin`, `urldefrag`, `urlsplit`, `urlunsplit`, `urlencode`, `parse_qs`,
  `parse_qsl`, `quote`, `quote_plus`, `quote_from_bytes`, `unquote`,
  `unquote_plus`, `unquote_to_bytes`.
- **Class (6 NamedTuples):** `DefragResult`, `ParseResult`, `SplitResult`,
  `DefragResultBytes`, `ParseResultBytes`, `SplitResultBytes`. All are
  6-tuple NamedTuples (scheme, netloc, path, params, query, fragment +
  helpers). `ParseResult` is 6-tuple; `SplitResult` is 5-tuple
  (no params).

## Hot-path classification — pure-scalar string compute

Every fn is **pure string in → string/tuple out**. No I/O, no callbacks,
no cycle-capable allocations (str is atomic per #2128 refinement). Hot
patterns:
- `urlparse(s)` — single-pass scan-and-slice; returns a 6-tuple-shaped
  NamedTuple. Tuple alloc is the only friction.
- `quote(s)` / `unquote(s)` — char-by-char encode/decode. ~3-byte
  output per %XX in `quote`; `unquote` is shorter. Pure scan-and-build.
- `urlencode(dict)` — iterate + `quote_plus` each k/v + join. Dict
  iteration overhead but no callbacks.

## Predicted regime — balanced (compute-leaning)

- 100k iter of `urlparse("https://example.com/a/b?x=1&y=2#f")`:
  ~80-120 ms in CPython (C-accel path lives in `_urlparse` C ext).
  Mamba should compute-win on the branch path + str slice; tuple
  alloc per call is the friction, mitigated by #2128 atomic-only
  fast path.
- 100k iter of `quote("hello world/?&")`:
  ~30-50 ms in CPython. Mamba should crush this — pure char loop.
- 100k iter of `urlencode({"a":"1","b":"2","c":"3"})`:
  Dict iter + 3× quote_plus + join. ~80-120 ms in CPython.

**Expected G2:** wall 5-15× across the fixtures, internal 2-6×. Mem
ratio likely PASS (no large-bytes materialization; #2096 subset A/B
don't apply to short URLs).

**Tier:** **compute** for quote/unquote, **balanced** for urlparse/urlencode.

## Blockers / carve-outs

- **6-tuple NamedTuple return**: ParseResult is the trickiest piece.
  Options: (a) return plain tuple (lose `.scheme` attribute access —
  CPython conformance miss); (b) return Instance with both indexed
  AND named access (full conformance, more LOC); (c) integer-handle
  pattern (overkill — these are short-lived, no shared mutation).
  Recommendation: (b) Instance with both index + name access,
  matching the colorsys/divmod tuple-return pattern but extended
  with a per-instance attribute fallback in `mb_getattr`.
- **Bytes overloads**: `quote_from_bytes`, `unquote_to_bytes`,
  `DefragResultBytes` etc. handle bytes input. Subset A risk if
  benched on multi-MB blobs — but standard URL workloads are short
  strings, so realistic benches stay sub-#2096-threshold.
- **No callbacks**: no #2100 risk.
- **No operator overloads**: no #2129 risk.

## Bench sketch

```python
# bench/urlparse_bulk.py — tier: compute
import urllib.parse, sys, time
_urlparse = urllib.parse.urlparse  # #2097 hoist
ITERS = 100_000
acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    r = _urlparse("https://example.com/a/b?x=1&y=2#f")
    acc += len(r.path)
_t1 = time.perf_counter()
print("urlparse_bulk:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

Second fixture: `quote_bulk.py` — 100k `quote("hello world/?&")` on
a short ASCII string. Pure compute fingerprint.

## LOC / G2 estimate

- **Estimated LOC**: ~280 (mostly quote/unquote table-driven char
  loops; urlparse is a ~50-line scan; rest are thin wrappers).
- **Expected G2**: wall 5-15× PASS, internal 2-6× PASS, mem 0.9-1.1×
  borderline (NamedTuple per-call Instance allocation if path (b)).

## Ship-now confidence: **HIGH** (wave-4 leader)

Rationale: no #2100, no #2129, no #2096 subset A risk on realistic
workloads, pure compute hot path. Highest perf-PASS confidence in
wave-4. NamedTuple ABI is the only design call.
