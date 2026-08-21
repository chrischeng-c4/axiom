# Operational AssertionPass seed for SILENT divergences across the
# codec-registry / structured-CSV pair pinned by atomic 169:
# `codecs` (the documented `lookup(name).name` round-trip
# contract + the documented `encode(...)` transform-codec
# surface for hex / base64 / rot_13) and `csv` (the documented
# `writer(buf).writerow(...)` instance method + the documented
# `reader(buf)` iteration contract).
#
# The matching subset (codecs UTF-8 encode/decode layer + BOM
# constant layer + module hasattr surface, base64 full encoder
# layer, gzip compress / decompress round-trip layer, zlib full
# layer, json nested-container layer, csv module hasattr
# surface) is covered by
# `test_codecs_base64_gzip_zlib_json_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • codecs.lookup("utf-8").name == "utf-8" — documented
#     CodecInfo `.name` attribute round-trip (mamba: returns
#     None — lookup() return doesn't surface a CodecInfo
#     instance with the documented attribute);
#   • codecs.encode(b"hi", "hex") == b"6869" — documented hex
#     transform codec (mamba: returns None — transform codec
#     unregistered);
#   • codecs.encode(b"hi", "base64") == b"aGk=\n" — documented
#     base64 transform codec (mamba: returns None);
#   • codecs.encode("hello", "rot_13") == "uryyb" — documented
#     rot_13 transform codec (mamba: returns b"hello" — the
#     input bytes pass through unchanged);
#   • csv.writer(buf).writerow(["a", "b"]) populates buf to
#     contain "a,b\r\n" — documented Writer instance method
#     (mamba: AttributeError 'str' object has no attribute
#     'writerow' — csv.writer return type is broken);
#   • list(csv.reader(io.StringIO("a,b\n1,2\n"))) == [["a","b"],
#     ["1","2"]] — documented Reader iteration contract
#     (mamba: returns the empty list — reader generator never
#     yields).
import codecs as _codecs_mod
import csv as _csv_mod
import io
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# transform codecs / Writer instance method / Reader iterator
# return types that mamba's bundled type stubs do not surface
# accurately.
codecs: Any = _codecs_mod
csv: Any = _csv_mod


_ledger: list[int] = []

# 1) codecs.lookup — CodecInfo .name round-trip
assert codecs.lookup("utf-8").name == "utf-8"; _ledger.append(1)

# 2) codecs.encode — transform codec surface
assert codecs.encode(b"hi", "hex") == b"6869"; _ledger.append(1)
assert codecs.encode(b"hi", "base64") == b"aGk=\n"; _ledger.append(1)
assert codecs.encode("hello", "rot_13") == "uryyb"; _ledger.append(1)

# 3) csv.writer — Writer instance .writerow contract
_buf = io.StringIO()
_w = csv.writer(_buf)
_w.writerow(["a", "b"])
assert _buf.getvalue() == "a,b\r\n"; _ledger.append(1)
_w.writerow([1, 2])
assert _buf.getvalue() == "a,b\r\n1,2\r\n"; _ledger.append(1)

# 4) csv.reader — Reader iteration contract
_rows = list(csv.reader(io.StringIO("a,b\n1,2\n")))
assert _rows == [["a", "b"], ["1", "2"]]; _ledger.append(1)
assert len(_rows) == 2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_codecs_csv_silent {sum(_ledger)} asserts")
