# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `b'abc'.upper()` (the documented
# "bytes.upper returns uppercased bytes" — mamba raises Attribute
# Error: bytes object has no attribute 'upper'), `b'AbC'.lower()`
# (the documented "bytes.lower returns lowercased bytes" — mamba
# raises AttributeError), `b'abc'.title()` (the documented "bytes.
# title returns title-cased bytes" — mamba raises AttributeError),
# `b'AbC'.swapcase()` (the documented "bytes.swapcase swaps case" —
# mamba raises AttributeError), `b'abc'.capitalize()` (the
# documented "bytes.capitalize capitalizes first byte" — mamba
# raises AttributeError), `b'abc'.center(7)` (the documented "bytes.
# center pads bytes to width" — mamba raises AttributeError),
# `b'abc'.ljust(5)` (the documented "bytes.ljust left-aligns bytes"
# — mamba raises AttributeError), `b'abc'.isalpha()` (the documented
# "bytes.isalpha returns True for all-alpha bytes" — mamba raises
# AttributeError), `2 ** -1` (the documented "int ** negative int
# returns float (0.5)" — mamba returns 0 silently), and `0 ** -1`
# (the documented "0 ** negative int raises ZeroDivisionError" —
# mamba returns 0 silently).
# Ten-pack pinned to atomic 323.
#
# Behavioral edges that CONFORM on mamba (list count/index/append/
# extend/insert/remove/pop/sort/reverse/copy. dict keys/values/
# items/get/setdefault/update/pop/clear/copy/'in'. set add/remove/
# discard/union/intersection/difference/symmetric_difference/issub
# set/issuperset/isdisjoint. str title/lower/upper/swapcase/
# capitalize/casefold/count/find/rfind/ljust/rjust/center/zfill/
# is*/partition/rpartition/rsplit/splitlines/lstrip/rstrip/strip-
# chars/startswith-tuple/endswith-tuple/removeprefix/removesuffix/
# replace-count/find-with-bounds. bytes split/join/startswith/
# replace/hex/fromhex/count/find. iter/next/StopIteration. divmod
# and floor-div/modulo on negatives. complex numbers. Container
# exception protocols: list.index/remove ValueError, set.remove
# KeyError, dict.pop/dict[k] KeyError, next() StopIteration) are
# covered in the matching pass fixture
# `test_lang_collection_methods_value_ops`.


_ledger: list[int] = []

# 1) b'abc'.upper() returns b'ABC'
#    (mamba: raises AttributeError — method missing)
assert b"abc".upper() == b"ABC"; _ledger.append(1)

# 2) b'AbC'.lower() returns b'abc'
#    (mamba: raises AttributeError — method missing)
assert b"AbC".lower() == b"abc"; _ledger.append(1)

# 3) b'abc'.title() returns b'Abc'
#    (mamba: raises AttributeError — method missing)
assert b"abc".title() == b"Abc"; _ledger.append(1)

# 4) b'AbC'.swapcase() returns b'aBc'
#    (mamba: raises AttributeError — method missing)
assert b"AbC".swapcase() == b"aBc"; _ledger.append(1)

# 5) b'abc'.capitalize() returns b'Abc'
#    (mamba: raises AttributeError — method missing)
assert b"abc".capitalize() == b"Abc"; _ledger.append(1)

# 6) b'abc'.center(7) returns b'  abc  '
#    (mamba: raises AttributeError — method missing)
assert b"abc".center(7) == b"  abc  "; _ledger.append(1)

# 7) b'abc'.ljust(5) returns b'abc  '
#    (mamba: raises AttributeError — method missing)
assert b"abc".ljust(5) == b"abc  "; _ledger.append(1)

# 8) b'abc'.isalpha() returns True
#    (mamba: raises AttributeError — method missing)
assert b"abc".isalpha() == True; _ledger.append(1)

# 9) 2 ** -1 returns 0.5 (float promotion on negative power)
#    (mamba: returns 0 silently — integer floor instead of float)
assert 2 ** -1 == 0.5; _ledger.append(1)

# 10) 0 ** -1 raises ZeroDivisionError
#     (mamba: returns 0 silently — no division check)
try:
    0 ** -1
    raise AssertionError("expected ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_bytes_methods_negpow_silent {sum(_ledger)} asserts")
