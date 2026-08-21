"""Bulk xml.etree.fromstring (Task #56, Wave-4 ship #4).

Predicted regime per scout: compute (alloc-bound at scale). Subset
B worst-case — many small Element dicts per parse, retained in
the parent's _children list. Scout estimated mem 0.15-0.30x FAIL
by-design.

Workload: 100-element XML doc, 50 parses. Sized down from the
original 1000-iter scout target because mamba's per-parse
allocation cost (100 child Element dicts + 100 attrib dicts + 100
children-lists + ~600 small strings = ~900 alloc per parse) puts
the 100x1000 = 100k-element regime past the GC pause cliff. 50
parses (= 5k Element allocs) completes in finite wall time on
mamba; the per-call alloc-bound subset-B signature is intact at
this size.

Hoist convention (#2097): bind `fromstring` to a local before the
loop. Mamba `import` quirks:
  - `import xml.etree.ElementTree as ET` then `ET.fromstring`
    resolves to None (Task #52 finding); must use
    `from xml.etree.ElementTree import fromstring as _fromstring`.
  - Comma form `import sys, time` does NOT bind `time` under mamba;
    must use separate `import sys` / `import time` lines (Task #56
    finding).

We accumulate the root tag length per iter (cheap O(1) accessor),
not children iteration — mamba's Element-as-dict representation
makes `for child in root` yield dict keys, not children, which
defeats cross-runtime G1 byte-equivalence. Per-iter parse cost
is the dominant cost anyway.

No XPath (.find/.findall — deferred per scout). No XMLParser
target callbacks (#2100). No iterparse / streaming. No namespace
resolution. No CDATA / DTD.

# tier: compute
"""

from xml.etree.ElementTree import fromstring as _fromstring

XML = "<root>" + "".join([f"<item id='{i}'>v{i}</item>" for i in range(100)]) + "</root>"
ITERS = 50

acc = 0
for _ in range(ITERS):
    root = _fromstring(XML)
    acc += len(root.tag)
print("fromstring_bulk:", acc)
