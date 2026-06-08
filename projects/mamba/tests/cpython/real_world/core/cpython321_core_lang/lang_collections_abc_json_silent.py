# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_collections_abc_json_silent"
# subject = "cpython321.lang_collections_abc_json_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_collections_abc_json_silent.py"
# status = "filled"
# ///
"""cpython321.lang_collections_abc_json_silent: execute CPython 3.12 seed lang_collections_abc_json_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(collections.abc, 'Iterable')`
# (the documented "collections.abc exposes the Iterable ABC" — mamba
# returns False), `hasattr(collections.abc, 'Iterator')` (the
# documented "collections.abc exposes the Iterator ABC" — mamba
# returns False), `hasattr(collections.abc, 'Callable')` (the
# documented "collections.abc exposes the Callable ABC" — mamba
# returns False), `hasattr(collections.abc, 'Sequence')` (the
# documented "collections.abc exposes the Sequence ABC" — mamba
# returns False), `hasattr(collections.abc, 'Mapping')` (the
# documented "collections.abc exposes the Mapping ABC" — mamba
# returns False), `hasattr(collections.abc, 'Set')` (the documented
# "collections.abc exposes the Set ABC" — mamba returns False),
# `hasattr(json.encoder, 'JSONEncoder')` (the documented "json.encoder
# exposes the JSONEncoder class" — mamba returns False), `hasattr(
# json.encoder, 'py_encode_basestring')` (the documented "json.encoder
# exposes the py_encode_basestring helper" — mamba returns False),
# `hasattr(json.decoder, 'JSONDecoder')` (the documented "json.decoder
# exposes the JSONDecoder class" — mamba returns False), and
# `hasattr(json.decoder, 'JSONDecodeError')` (the documented "json.
# decoder exposes the JSONDecodeError exception" — mamba returns
# False).
# Ten-pack pinned to atomic 319.
#
# Behavioral edges that CONFORM on mamba (collections — hasattr deque/
# OrderedDict/defaultdict/Counter/ChainMap/namedtuple/UserDict/User
# List/UserString. heapq — hasattr heappush/heappop/heapify/heappush
# pop/heapreplace/nlargest/nsmallest/merge. bisect — hasattr bisect/
# bisect_left/bisect_right/insort/insort_left/insort_right. json —
# hasattr dumps/loads/dump/load/JSONEncoder/JSONDecoder/JSONDecode
# Error (top-level). functools — hasattr reduce/partial/partialmethod/
# cache/lru_cache/wraps/update_wrapper/singledispatch/singledispatch
# method/cached_property/cmp_to_key/total_ordering/WRAPPER_ASSIGNMENTS
# /WRAPPER_UPDATES. itertools — hasattr count/cycle/repeat/chain/
# compress/dropwhile/takewhile/islice/starmap/filterfalse/groupby/
# accumulate/tee/zip_longest/product/permutations/combinations/
# combinations_with_replacement/pairwise/batched) are covered in the
# matching pass fixture
# `test_collections_heapq_json_functools_itertools_value_ops`.
from collections import abc
from json import encoder as jenc, decoder as jdec


_ledger: list[int] = []

# 1) hasattr(collections.abc, 'Iterable') — Iterable ABC
#    (mamba: returns False)
assert hasattr(abc, "Iterable") == True; _ledger.append(1)

# 2) hasattr(collections.abc, 'Iterator') — Iterator ABC
#    (mamba: returns False)
assert hasattr(abc, "Iterator") == True; _ledger.append(1)

# 3) hasattr(collections.abc, 'Callable') — Callable ABC
#    (mamba: returns False)
assert hasattr(abc, "Callable") == True; _ledger.append(1)

# 4) hasattr(collections.abc, 'Sequence') — Sequence ABC
#    (mamba: returns False)
assert hasattr(abc, "Sequence") == True; _ledger.append(1)

# 5) hasattr(collections.abc, 'Mapping') — Mapping ABC
#    (mamba: returns False)
assert hasattr(abc, "Mapping") == True; _ledger.append(1)

# 6) hasattr(collections.abc, 'Set') — Set ABC
#    (mamba: returns False)
assert hasattr(abc, "Set") == True; _ledger.append(1)

# 7) hasattr(json.encoder, 'JSONEncoder') — JSONEncoder class in submodule
#    (mamba: returns False)
assert hasattr(jenc, "JSONEncoder") == True; _ledger.append(1)

# 8) hasattr(json.encoder, 'py_encode_basestring') — py_encode_basestring helper
#    (mamba: returns False)
assert hasattr(jenc, "py_encode_basestring") == True; _ledger.append(1)

# 9) hasattr(json.decoder, 'JSONDecoder') — JSONDecoder class in submodule
#    (mamba: returns False)
assert hasattr(jdec, "JSONDecoder") == True; _ledger.append(1)

# 10) hasattr(json.decoder, 'JSONDecodeError') — JSONDecodeError exception in submodule
#     (mamba: returns False)
assert hasattr(jdec, "JSONDecodeError") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_collections_abc_json_silent {sum(_ledger)} asserts")
