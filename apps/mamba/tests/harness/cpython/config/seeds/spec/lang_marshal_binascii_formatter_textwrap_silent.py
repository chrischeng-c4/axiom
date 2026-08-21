# Operational AssertionPass seed for SILENT divergences across the
# marshal.dumps return-type / marshal round-trip + binascii.crc32
# / Error / Incomplete extended module-helper surface +
# string.Formatter class-identity / instance .format value
# contract + textwrap.TextWrapper class instantiation pinned by
# atomic 184: `marshal` (the documented `marshal.dumps(x)` bytes
# return-type contract + the documented `marshal.loads
# (marshal.dumps(x)) == x` round-trip contract), `binascii` (the
# documented `crc32` function identifier + the documented `Error`
# / `Incomplete` exception identifiers + the documented
# `binascii.crc32(b"hello") == 907060870` integer value contract),
# `string` (the documented `type(string.Formatter()).__name__ ==
# "Formatter"` class-identity contract + the documented
# `string.Formatter().format(template, *args)` value contract),
# and `textwrap` (the documented `textwrap.TextWrapper(width=N)`
# class instantiation contract + the documented
# `TextWrapper.fill(...)` value contract).
#
# The matching subset (full pickle module hasattr surface +
# pickle round-trip + full marshal module hasattr surface +
# partial binascii module hasattr surface (a2b_hex / b2a_hex /
# a2b_base64 / b2a_base64 / hexlify / unhexlify) + binascii
# hexlify/unhexlify value contract + full base64 module
# hasattr surface + base64 b64/b32/b16/urlsafe encode-decode
# value contract + quopri encodestring/decodestring hasattr +
# string.Formatter/Template hasattr + full shutil module
# hasattr surface + shutil.get_terminal_size terminal_size
# return-type) is covered by
# `test_pickle_marshal_base64_binascii_shutil_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(marshal.dumps([1, 2.0, "hello"])).__name__ ==
#     "bytes" — documented return-type contract (mamba:
#     returns "str");
#   • marshal.loads(marshal.dumps([1, 2.0, "hello", None,
#     True])) == [1, 2.0, "hello", None, True] —
#     documented round-trip value contract (mamba: loads
#     returns None — the marshal payload is unparseable);
#   • hasattr(binascii, "crc32") is True — documented
#     function identifier (mamba: False);
#   • hasattr(binascii, "Error") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(binascii, "Incomplete") is True — documented
#     exception identifier (mamba: False);
#   • binascii.crc32(b"hello") == 907060870 — documented
#     integer value contract (mamba: raises AttributeError
#     'dict' object has no attribute 'crc32');
#   • type(string.Formatter()).__name__ == "Formatter" —
#     documented class-identity contract (mamba: returns
#     "dict" — the Formatter constructor returns a plain
#     dict not a Formatter instance);
#   • string.Formatter().format("hello {0} {1}", "world",
#     "!") == "hello world !" — documented value contract
#     (mamba: raises AttributeError on .format method);
#   • type(textwrap.TextWrapper(width=15)).__name__ ==
#     "TextWrapper" — documented class instantiation
#     contract (mamba: raises AttributeError on
#     .TextWrapper accessor — the class identifier is not
#     bound on the textwrap module).
import marshal as _marshal_mod
import binascii as _binascii_mod
import string as _string_mod
import textwrap as _textwrap_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance-method / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
marshal: Any = _marshal_mod
binascii: Any = _binascii_mod
string: Any = _string_mod
textwrap: Any = _textwrap_mod


_ledger: list[int] = []

# 1) marshal — return type + round-trip
_data_m = [1, 2.0, "hello", None, True]
_b_m = marshal.dumps(_data_m)
assert type(_b_m).__name__ == "bytes"; _ledger.append(1)
assert marshal.loads(_b_m) == _data_m; _ledger.append(1)

# 2) binascii — extended class / function / exception identifiers
assert hasattr(binascii, "crc32") == True; _ledger.append(1)
assert hasattr(binascii, "Error") == True; _ledger.append(1)
assert hasattr(binascii, "Incomplete") == True; _ledger.append(1)
assert binascii.crc32(b"hello") == 907060870; _ledger.append(1)

# 3) string.Formatter — class identity + format value contract
_fmt = string.Formatter()
assert type(_fmt).__name__ == "Formatter"; _ledger.append(1)
assert _fmt.format("hello {0} {1}", "world", "!") == "hello world !"; _ledger.append(1)

# 4) textwrap.TextWrapper — class instantiation + fill value
_tw = textwrap.TextWrapper(width=15)
assert type(_tw).__name__ == "TextWrapper"; _ledger.append(1)
_filled = _tw.fill("the quick brown fox jumps over the lazy dog")
assert _filled == "the quick brown\nfox jumps over\nthe lazy dog"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_marshal_binascii_formatter_textwrap_silent {sum(_ledger)} asserts")
