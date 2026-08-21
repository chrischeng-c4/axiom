"""Bulk mimetypes.guess_type (Task #64, Wave-5 ship #2).

Predicted regime per scout: compute (pure-string suffix lookup
against a baked TYPES_MAP table). Wall target >=3.0x because the
mamba shim resolves to a Rust slice-of-tuples scan while CPython
builds a 200+-entry Python dict at module import.

Workload: 10000 mixed-extension filenames, 10 iters.
Per scout sequencing: mimetypes ship is from-scratch surface (#1479);
this fixture pairs with mimetypes_mod.rs registering 6 dispatchers
plus the MimeTypes class shell at the same revision.

Hoist convention (#2097): bind `mimetypes.guess_type` locally.
Mamba import quirk avoidance (#2056): separate `import sys` /
`import time` / `import mimetypes` lines (xml.etree Task #56 finding).

# tier: compute
"""

import mimetypes

_guess = mimetypes.guess_type

EXTS = (".html", ".png", ".jpg", ".tar.gz", ".css", ".js", ".unknown")
NAMES = []
for i in range(10000):
    NAMES.append(f"file_{i}{EXTS[i % len(EXTS)]}")
ITERS = 10

acc = 0
for _ in range(ITERS):
    for name in NAMES:
        mtype, _enc = _guess(name)
        if mtype is not None:
            acc += 1
print("guess_type_bulk:", acc)
