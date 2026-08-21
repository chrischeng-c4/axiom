# mamba perf benches

Python-side micro-benchmarks comparing mamba vs CPython 3.12.
Each `.py` file is self-contained: both runtimes execute the
same script and emit `INTERNAL_TIME_NS=<ns>` on stderr; a
cross-runtime harness divides them to yield a ratio.

## Layout (DDD bounded contexts)

```
benches/
  3p/                 # Rust criterion harnesses (existing)
  mamba_bench.rs      # Rust top-level harness (existing)

  language_bench/     # Python language-core probes (55 files / 11 modules)
    arithmetic/       # int_add, int_mul, int_div, int_mod, float_add,
                      #   float_sub, float_mul, float_div
    bytes/            # find, replace, split, startswith_endswith, index
    classes/          # attr_read, attr_write
    comprehensions/   # listcomp_square, listcomp_filter, dictcomp
    exceptions/       # try_no_raise                 [tick-183 new ctx]
    functions/        # call_overhead, kwargs_call
    integers/         # bit_length_count, from_bytes, to_bytes, abs_neg
    mappings/         # dict_get, dict_update, dict_pop, dict_items_iter,
                      #   dict_setdefault, dict_method_get_default,
                      #   dict_keys_iter
    sequences/        # list_index, list_append_loop, list_sort_key,
                      #   list_pop, list_method_index, tuple_unpack,
                      #   tuple_count, tuple_method_index
    sets/             # intersection, union, frozenset_intersection
    strings/          # concat, count, split, splitlines, partition, join,
                      #   replace, format, isalpha_isdigit, zfill,
                      #   upper_lower, strip

  builtins_bench/     # Python builtins, one dir per builtin (30 files / 25 modules)
    abs/  any_all/  bin_hex_oct/  bool/  chr_ord/  divmod/  enumerate/
    filter/  float_parse/  frozenset/  hash/  int_parse/  isinstance/
    len/  map/  max_min/  pow/  range/  repr/  reversed/  round/
    sorted/  str_conv/  sum/  zip/

  pep_bench/          # One dir per PEP being probed (17 files / 16 modules)
    pep318_decorators/  pep343_with/  pep380_yield_from/
    pep448_unpack_generalize/  pep484_type_hints/  pep492_async/
    pep498_fstrings/ (+1 fstring_mixed)  pep526_var_annot/
    pep557_dataclasses/ pep572_walrus/  pep604_union/  pep634_match/
    pep654_except_star/ pep3104_nonlocal/  pep3119_abc/
    pep3132_star_unpack/

  stdlib_bench/       # Python stdlib hot paths (78 files / 28 modules)
    base64/  binascii/  bisect/  bytes/  collections/
    colorsys/  copy/  fnmatch/  functools/  gzip/  hashlib/
    heapq/  hmac/  itertools/  json/  keyword/  math/  os/
    pickle/  random/  re/  secrets/  statistics/  string/
    struct/  unicodedata/  urllib/  zlib/
```

Total: **180 perf benches** across 4 DDD bounded contexts (as of
tick-200 reorg checkpoint, 2026-05-27). Up +49 since tick-150
(+30 language-core, +1 builtins, +1 PEP, +17 stdlib).

## Bench file template

```python
"""<name> — <one-line scenario>.

Bounded context (DDD): <context>/<topic>.
Tier: compute | startup | balanced | allocation.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist module attrs to locals before the hot loop.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars (`outer`/`inner`).
"""
import sys, time
# Pre-bind module-level free fns (#2097); literal N / ITERS (mamba force-typed).
# DO NOT hoist bound methods of objects (e.g. _search = pat.search) —
# under mamba the hoisted ref returns None silently.
_t0 = time.perf_counter()
for outer in range(ITERS):
    ...  # hot body
_t1 = time.perf_counter()
print("<name>:", acc)                                # before marker (#2105)
print(f"INTERNAL_TIME_NS={int((_t1-_t0)*1_000_000_000)}",
      file=sys.stderr, flush=True)
diff = acc - expected                                # subtraction, not ==
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
```

## Conventions

- One file = one scenario = one atomic commit. No omnibus benches.
- Checksums use subtraction (`acc - expected`), not `==`, because of
  mamba's boxed/accumulator int-equality quirk.
- For float accumulators, use `math.isclose(...)`.
- For workloads where mamba allocates per iter (e.g. `*rest` unpack),
  cap `ITERS` so total runtime stays under ~3 s on both runtimes.
- `await` cannot appear mid-expression on mamba — bind to a temp first.
- Sanity-check any mamba "N× FASTER" result by probing the actual
  output — if mamba returns a trivially short/constant value, the
  module may be stubbed (see Known runtime gaps below) and the
  perf "win" is bogus.
- **Always check mamba's exit code** after measuring — `None`
  arithmetic propagates silently and can bypass `math.isclose` /
  subtract-checksum assertions. The tick-118 audit found **18 of 43**
  stdlib benches were silently broken because the measurement script
  only grepped `INTERNAL_TIME_NS` and ignored the assertion error.

## Mamba write-once quirks (always trip benches)

- **Nested `for _ in range(...)` loops** corrupt the outer `_` binding.
  Inner loop completes once or thousands of times depending on shape.
  Always use named loop vars (`outer`/`inner`). Found tick 126.
- **Hoisted bound methods** (`_m = obj.method; _m(...)`) return `None`
  under mamba for `re.Pattern.search`, `random.Random.randint`, and
  likely other handle-backed objects. Call `obj.method(...)` inline.
  Found ticks 119 and 126.
- `import urllib.parse` → `urllib.parse.fn` resolves to `None`.
  Use `from urllib.parse import fn` instead.
- `import a, b` → only `a` is bound; `b` stays undefined.
  Use separate `import` lines.
- `import datetime; datetime.datetime.now` → `None` (datetime is
  stubbed as a lambda — see below).

## Known runtime gaps (don't bench against these directly)

mamba returns `None` / `0` / garbage from these stdlib paths but
*propagates silently* — `None - 6.0 → None`, `abs(None) < 1e-9 → True`,
so float checksums pass under mamba even when the work didn't happen.
**Always probe with `mamba run` of a 2-line script before writing a
bench, and check mamba's exit code after measurement.**

Stubbed / broken under mamba (tick-200 audit, 2026-05-27):

Newly observed (ticks 151-200, all discarded as bench candidates):

- **`operator.itemgetter`** — callable returns `0` not `row[i]`
  (tick 172 candidate).
- **`os.path.normpath`** — returns `None`/empty string (tick 174 candidate).
- **`collections.deque.extendleft(batch)`** — leaves `len(dq)` at 0,
  either extendleft clears or len-of-deque is broken (tick 187 candidate).
- **`bytes.upper/lower`** — `AttributeError: 'bytes' has no attribute
  'upper'` (tick 165 candidate, hard-fails cleanly).
- **`csv.reader`** — produces no rows (tick 192 candidate).
- **`enum.IntEnum`** — member compare returns 0 (tick 188 candidate).

Pre-tick-150 list (still applies):

- **`datetime`** — `<lambda>` factory; class methods → `None`.
- **`traceback.format_exc`** — literal `"NoneType: None\n"`, no walk.
- **`array.array("i"/"d")`** — `len==0`, iter raises silently, `[i]==None`.
- **`bisect.insort`** — does insort but `len(sorted_list)` returns 0.
- **`csv.reader`** — yields `[]` (no rows produced).
- **`configparser`, `logging`, `subprocess`, `tempfile`, `argparse`,
  `html`, `tarfile`, `glob`, `weakref`** — all are `dict` stubs.
- **`collections.deque`** — `len(d)` returns 0 (append/popleft/iter work).
- **`collections.OrderedDict.move_to_end`** — `AttributeError` (missing).
- **`collections.Counter.update(list)`** — increments stay 0; `update(dict)`
  works but item-list path is broken (tick 131).
- **`dataclasses`** — `@dataclass` instance attrs return `None`.
- **`decimal.Decimal`** — arithmetic returns garbage int (overflow).
- **`enum.IntEnum`** — member lookup returns 0.
- **`io.StringIO.write`** — write/getvalue silently no-op.
- **`operator.itemgetter`** — callable returns `None`.
- **`pathlib.PurePath`** — `str(p)` returns `<PurePosixPath instance>` repr.
- **`queue.Queue.get`** — returns `None`.
- **`re.Pattern.match`** — always-None match.
- **`shlex.split`** — doesn't handle quotes (raw whitespace split).
- **`string.Template`** — `dict` stub, no `.substitute()`.
- **`struct.unpack`** — works but 48-bit int overflow at 2^47 in checksum sums.
- **`textwrap.fill`** — returns input string unchanged (no wrap).
- **`time.time_ns`** — returns `None`.
- **`typing.NamedTuple`** — instance attr returns `None`.
- **`uuid.uuid4().int`** — garbage value.
- **`os.path.X` via `import os.path`** — resolves to `None`; use
  `from os.path import X`.

## Tick-150 results snapshot (verified-real benches, 2026-05-26)

**Mamba FASTER (lower ratio = better):**

| Bench | Ratio | Speedup |
|------|------|---------|
| `statistics.mean` | 0.063× | 16× faster |
| `random.shuffle` | 0.126× | 7.9× faster |
| `urllib.parse.urlparse` | 0.243× | 4.1× faster |
| `min/max(key=)` | 0.44× | 2.3× faster |
| `os.path.basename+dirname` | 0.46× | 2.15× faster |
| `random.choice` | 0.458× | 2.2× faster |
| `list.sort(key=)` | 0.50× | 2× faster |
| `os.path.splitext` | 0.60× | 1.7× faster |
| `json.loads` (nested list) | 0.72× | 1.4× faster |
| `str.join` | 0.74× | 1.33× faster |
| `secrets.token_hex` | 0.778× | 1.3× faster |
| `urllib.parse.quote` | 0.811× | 1.2× faster |
| `math.sin+cos` | 0.85× | 1.18× faster |
| `int(str)` parse | 0.90× | parity / slight |
| `heapq.nlargest` | 0.907× | 1.1× faster |
| `str.count` | 0.95× | parity |
| `base64.b64encode` | 0.990× | parity |
| `zlib.crc32` | 1.007× | parity |
| `base64.b64decode` | 1.062× | parity |
| `math.sqrt` | 1.105× | parity |

**Mamba SLOWER (notable regressions):**

| Bench | Ratio | Slowdown |
|------|------|----------|
| `dict.setdefault.append` | ~107× | per-iter list alloc + GC threshold |
| `struct.pack` | 97.0× | format-string reparse per call |
| `str.isalpha/isalnum/isdigit` | ~80× | per-string-method dispatch |
| `re.sub` | 72.8× | per-match new_str + Pattern reparse |
| `bytes.hex` method | 70.1× | class.rs dispatch tax (use binascii.hexlify) |
| `functools.partial` call | ~70× | partial wrapper rebuilds args tuple per call |
| `int.bit_length+bit_count` | ~60× | per-int-method dispatch vs C popcount |
| `set.intersection` | ~49× | overlap alloc + hash probe per pair |
| `collections.deque` append/popleft | ~40× | per-call bound-method dispatch |
| `collections.defaultdict` group-by | ~39× | default-factory dispatch + list.append tax |
| `frozenset` build + `in` | ~30× | construction + per-probe contains tax |
| `list.append` (hot) | ~24× | per-call dispatch through Vec |
| `math.atan2` | ~17× | libm path not as well-optimised as sin/cos |
| `colorsys.rgb_to_hsv` | 14.8× | tuple-return alloc (regression from 11.5× faster) |
| `dict.pop(default)` | ~13× | dict construct + per-key mutation |
| `dict.update` bulk-merge | ~11× | resize-aware merge tax |
| `str.split` tokenise | ~11× | list-of-str alloc per call |
| `bytes.find` scan loop | ~10.6× | C memmem-style faster |
| `bisect.bisect_left` | ~9× | per-call dispatch through typed bridge |
| `re.findall` | 8.4× | per-match new_str alloc |
| `unicodedata.normalize` | 8.0× | per-char new_str on canonicalisation |
| `pickle.loads` | 7.0× | per-key MbObject during dict reconstruct |
| `random.randint` (seeded) | ~6.5× | per-call dispatch + box int |
| `heapq.heappush+heappop` | 6.3× | per-call dispatch + sift up/down |
| `functools.reduce` | 5.8× | operator.add wrapper dispatch per step |
| `sum(genexp)` | 5.1× | PyObject_Next + fold |
| `dict.items()` iter | 4.2× | per-pair tuple-yield + unpack tax |
| `itertools.groupby` | 4.0× | per-group handle dispatch + key compare |
| `hmac.new(sha256)` | 3.8× | HMAC handle wrap + SHA256 internal call |
| `bytes.replace` rewrite | 3.1× | per-call new bytes alloc |
| `str.translate` | 3.0× | per-call MbObject(str) return alloc |
| `fnmatch.fnmatch` | 2.6× | regex-engine invocation tax |
| `keyword.iskeyword` | 2.6× | frozenset contains wrapper tax |
| `tuple.count` | 2.4× | per-element compare + box |
| `hashlib.blake2b` | 2.4× | hash setup tax per call |
| `gzip.decompress` | 2.3× | bytes output alloc |
| `collections.OrderedDict` set | 2.3× | per-key dict insert tax |
| `itertools.combinations` | 2.2× | per-tuple yield alloc |
| `pickle.dumps` | 2.1× | per-call Pickler setup |
| `itertools.product` | 2.1× | per-pair tuple-yield alloc |
| `binascii.hexlify` | 1.9× | per-call MbObject(bytes) alloc |
| `collections.Counter.most_common` | 1.8× | per-char Counter build |
| `math.log` | 1.8× | per-call libm dispatch heavier than sqrt |
| `chr/ord` builtins | ~1.6× | per-call codepoint conversion box |
| `itertools.accumulate` | ~1.6× | per-call accumulator wrap |
| `float(str)` parse | ~1.5× | strtod path slightly slower |
| `gzip.compress` | 1.5× | bytes output alloc |
| `repr(obj)` mixed | ~1.4× | tp_repr slot dispatch + box result |

## Tick-200 results snapshot (verified-real benches, 2026-05-27)

**New FASTER wins since tick 150 (selected, ratio = mamba/cpython):**

| Bench | Ratio | Speedup |
|------|------|---------|
| `int %` (mod hot)                         | 0.038× | 26.08× faster |
| `int //` (div hot)                        | 0.146× | 6.86× faster  |
| `float /`                                 | 0.139× | 7.2× faster   |
| `copy.deepcopy(nested list)`              | 0.184× | 5.44× faster  |
| `float *`                                 | 0.177× | 5.7× faster   |
| `float -`                                 | 0.185× | 5.4× faster   |
| `int abs/-neg`                            | 0.260× | 3.85× faster  |
| `kwargs call (3-kwarg)`                   | 0.290× | 3.45× faster  |
| `math.floor/ceil`                         | 0.350× | 2.86× faster  |
| `math.log10/log2`                         | 0.373× | 2.68× faster  |
| `str.upper/lower`                         | 0.422× | 2.37× faster  |
| `bytes[i]` per-byte index                 | 0.493× | 2.03× faster  |
| `class.attr write` (existing slot)        | 0.543× | 1.84× faster  |
| `json.dumps(nested list)`                 | 0.605× | 1.65× faster  |
| `try/except no-raise`                     | 0.640× | 1.56× faster  |
| `bin/hex/oct(int)`                        | 0.943× | 1.06× faster  |

**New SLOWER regressions since tick 150 (selected):**

| Bench | Ratio | Slowdown |
|------|------|----------|
| `re.match` (anchored)                     | ~727×  | per-match alloc + Pattern reparse |
| `functools.lru_cache` hit                 | 30.8×  | wrapper dispatch + cache probe |
| `int.to_bytes` serialize                  | 29.8×  | per-call new-bytes alloc |
| `int.from_bytes` parse                    | 18.7×  | per-call new-PyLong alloc |
| `list.pop` tail drain                     | 18.8×  | per-pop box overhead |
| `set.union`                               | 18×    | union alloc + hash rebuild |
| `itertools.starmap`                       | 13×    | per-call tuple-unpack + dispatch |
| `frozenset.intersection`                  | 9.9×   | similar to set.intersection |
| `tuple.method-index`                      | 9.3×   | per-step compare + box |
| `dictcomp` (precomputed-val)              | 5.3×   | per-iter dict alloc + per-key insert |
| `bisect.insort`                           | 5.5×   | (also stubbed-len bug in stdlib_bench) |
| `str.splitlines`                          | 4.6×   | new-list + per-line new-str alloc |
| `str.strip`                               | 4.9×   | per-call new-str slice alloc |
| `itertools.repeat → take`                 | 3.9×   | per-call MbObject wrap |
| `dict.keys() iter`                        | 3.05×  | per-iter slot walk overhead |
| `language.listcomp + if filter`           | 6.4×   | per-kept-elem ref-add |
| `string.ascii_letters in c`               | 2.35×  | per-`in` linear scan |
| `str.partition`                           | 2.3×   | 3-tuple + 3 new-str slice alloc |
| `hashlib.md5(short)`                      | 2.1×   | hash setup tax per call |
| `secrets.compare_digest`                  | 1.6×   | per-call dispatch overhead |
| `struct.unpack(int)`                      | 1.48×  | per-call format parse |
| `pep498 fstring(mixed)`                   | 1.45×  | per-call temp-list + str-join |
| `random.uniform`                          | ~49×   | per-call native-handle dispatch |

## Tick cadence

- Each new bench = one atomic commit (`perf(mamba/benches): add X`).
- Every ~10 ticks: rebase onto `origin/project-mamba` + force-push
  (with explicit user approval per cycle).
- Every 50 ticks: structural reorg checkpoint (this README) + PR to
  `project-mamba` for merge (per directive #9).
- Commit messages **do not** include the Claude trailer.
