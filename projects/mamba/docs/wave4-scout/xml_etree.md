# xml.etree.ElementTree — subset B many-elements, allocation-bound

**Source:** `projects/mamba/vendor/typeshed/stdlib/xml/etree/ElementTree.pyi` (385 lines).

## Surface

- **Module-level def (~12):** `iselement`, `canonicalize`, `SubElement`,
  `Comment`, `ProcessingInstruction`, `register_namespace`, `tostring`,
  `tostringlist`, `dump`, `indent`, `parse`, `iterparse`, `fromstring`,
  `fromstringlist`, `XML`, `XMLID`.
- **Class (~9):** `Element` (the core type), `ElementTree`, `QName`,
  `ParseError`, `XMLParser`, `XMLPullParser`, `TreeBuilder`,
  `C14NWriterTarget`, `_IterParseIterator` (Protocol).

## Hot-path classification — **subset B many-elements** (per `phase2_crosscutting_blockers`)

Per memory: xml is pre-classified as subset B with **many small
Element objects** per parse. A 1MB XML doc with 10k elements creates
10k Element Instance allocations + each element's attrib dict + tag
str + text str + tail str. **Mamba's per-object header overhead
amplifies this 4-6×** vs CPython's PyObject_Malloc arena.

- Hot path: `ElementTree.fromstring(xml_blob)` builds a tree of N
  Element objects. Each Element holds `.tag` (str), `.attrib` (dict),
  `.text` (str|None), `.tail` (str|None), `.children` (list[Element]).
- Per-Element footprint estimate: mamba ~160 bytes vs CPython ~80
  bytes. At 10k Elements: 1.6 MB vs 0.8 MB just in headers.

## Predicted regime — allocation-bound, mem worst-case

**Worst of the wave-4 four** for memory ratio. Subset B at scale
across BOTH the parser allocation step AND the retention in the tree.
Expect:
- Parse 1 MB / 10k-element XML: mamba wall ~2-5× CPython (compute
  PASS), but mem **0.15-0.30× FAIL** by-design — worst subset B
  signature yet seen (worse than json 0.31× because Element has more
  fields than json dict entries).
- Tree iteration (`for elem in tree.iter()`): pure traversal, no new
  allocations. Wall PASS.

**Tier:** **compute** (parse is pure-state-machine — no callbacks).

**Expected G2:** wall 1.5-4×, internal 0.8-3×, **mem 0.15-0.30× FAIL
by-design per #2096 subset B**.

## Blockers / carve-outs

- **#2096 subset B (mem FAIL by-design)**: documented carve-out.
  Worst-case in the wave; ship anyway with explicit "subset B
  many-elements" carve.
- **XPath-like `.find()` / `.findall()` / `.iterfind()`**: parses a
  XPath subset. Implementation is recursive walk + pattern match;
  no callbacks. Forward ship of basic XPath (`tag`, `*`, `.//tag`)
  is feasible; full XPath 1.0 deferred.
- **XMLParser callbacks** (`start`, `end`, `data`, `comment` target
  fns): #2100 callback territory. **Carve out**: forward ship of
  `fromstring`/`tostring`/`parse` with the default tree-builder ONLY.
  Custom XMLParser targets raise NotImplementedError.
- **iterparse**: streaming parse with events. Iterator protocol +
  callback events. Deferred.
- **Namespace handling**: `register_namespace` + `{uri}localname`
  notation. Forward ship treats namespace as opaque string prefix;
  full namespace resolution deferred.
- **Comment / ProcessingInstruction**: thin Element subclasses.
  Include in v1.

## Bench sketch

```python
# bench/fromstring_bulk.py — tier: compute (mem FAIL by-design per #2096 B)
import xml.etree.ElementTree as ET
import sys, time
_fromstring = ET.fromstring  # #2097 hoist
XML = "<root>" + "".join([f"<item id='{i}'>v{i}</item>" for i in range(100)]) + "</root>"
ITERS = 1_000
acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    root = _fromstring(XML)
    for child in root:
        acc += len(child.text)
_t1 = time.perf_counter()
print("fromstring_bulk:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

## LOC / G2 estimate

- **Estimated LOC**: ~520 (XML state-machine parser ~250, Element class
  + tree builder ~150, tostring serializer ~80, XPath subset ~40).
  Largest in wave-4.
- **Expected G2**: wall 1.5-4× PASS likely, internal 0.8-3× PASS, **mem
  0.15-0.30× FAIL by-design**.

## Ship-now confidence: **LOW-MEDIUM** (wave-4 #4 or defer to wave-5)

Rationale: worst memory ratio in the wave (subset B many-elements),
largest LOC budget, most carve-outs (XPath, callbacks, namespaces,
iterparse, custom XMLParser target). The "ship the surface, perf
gate fails cleanly, document the carve" precedent applies — but the
list of carve-outs is so long that a half-broken surface is
arguably worse than no surface. **Recommend deferring full ship to
wave-5** and including only `Element` + `fromstring` + `tostring`
basic forms in wave-4 if budget allows after the other three lands.
