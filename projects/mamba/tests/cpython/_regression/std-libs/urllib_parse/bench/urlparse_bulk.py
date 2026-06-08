"""Bulk urllib.parse.urlparse(url) hot loop (Task #52, Wave-4 ship #1).

Predicted regime per scout doc: balanced (compute-leaning). Each iter
parses a fixed URL via a single-pass scan-and-slice. Returns a
ParseResult Instance (mamba) / NamedTuple (CPython) with named-attr
access for `.path`.

No #2100 callbacks; no #2129 operator overloads; no #2096 subset A
(no large-bytes materialization); subset B doesn't apply to short
URLs. Tier: compute.

Hoist convention (#2097): bind `urlparse` to a local before the loop
so each iter is a direct call.

# tier: compute
"""

from urllib.parse import urlparse as _urlparse

ITERS = 100_000

acc = 0
for _ in range(ITERS):
    r = _urlparse("https://example.com/a/b?x=1&y=2#f")
    acc += len(r.path)
print("urlparse_bulk:", acc)
