# Atomic 319 pass conformance — collections module (hasattr deque/
# OrderedDict/defaultdict/Counter/ChainMap/namedtuple/UserDict/User
# List/UserString) + heapq module (hasattr heappush/heappop/heapify/
# heappushpop/heapreplace/nlargest/nsmallest/merge) + bisect module
# (hasattr bisect/bisect_left/bisect_right/insort/insort_left/insort_
# right) + json module (hasattr dumps/loads/dump/load/JSONEncoder/
# JSONDecoder/JSONDecodeError) + functools module (hasattr reduce/
# partial/partialmethod/cache/lru_cache/wraps/update_wrapper/single
# dispatch/singledispatchmethod/cached_property/cmp_to_key/total_
# ordering/WRAPPER_ASSIGNMENTS/WRAPPER_UPDATES) + itertools module
# (hasattr count/cycle/repeat/chain/compress/dropwhile/takewhile/
# islice/starmap/filterfalse/groupby/accumulate/tee/zip_longest/
# product/permutations/combinations/combinations_with_replacement/
# pairwise/batched).
# All asserts match between CPython 3.12 and mamba.
import collections
import heapq
import bisect
import json
import functools
import itertools


_ledger: list[int] = []

# 1) collections — hasattr core surface
assert hasattr(collections, "deque") == True; _ledger.append(1)
assert hasattr(collections, "OrderedDict") == True; _ledger.append(1)
assert hasattr(collections, "defaultdict") == True; _ledger.append(1)
assert hasattr(collections, "Counter") == True; _ledger.append(1)
assert hasattr(collections, "ChainMap") == True; _ledger.append(1)
assert hasattr(collections, "namedtuple") == True; _ledger.append(1)
assert hasattr(collections, "UserDict") == True; _ledger.append(1)
assert hasattr(collections, "UserList") == True; _ledger.append(1)
assert hasattr(collections, "UserString") == True; _ledger.append(1)

# 2) heapq — hasattr (conformant subset)
assert hasattr(heapq, "heappush") == True; _ledger.append(1)
assert hasattr(heapq, "heappop") == True; _ledger.append(1)
assert hasattr(heapq, "heapify") == True; _ledger.append(1)
assert hasattr(heapq, "heappushpop") == True; _ledger.append(1)
assert hasattr(heapq, "heapreplace") == True; _ledger.append(1)
assert hasattr(heapq, "nlargest") == True; _ledger.append(1)
assert hasattr(heapq, "nsmallest") == True; _ledger.append(1)
assert hasattr(heapq, "merge") == True; _ledger.append(1)

# 3) bisect — hasattr
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)

# 4) json — hasattr (top-level conformant subset)
assert hasattr(json, "dumps") == True; _ledger.append(1)
assert hasattr(json, "loads") == True; _ledger.append(1)
assert hasattr(json, "dump") == True; _ledger.append(1)
assert hasattr(json, "load") == True; _ledger.append(1)
assert hasattr(json, "JSONEncoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecodeError") == True; _ledger.append(1)

# 5) functools — hasattr core surface
assert hasattr(functools, "reduce") == True; _ledger.append(1)
assert hasattr(functools, "partial") == True; _ledger.append(1)
assert hasattr(functools, "partialmethod") == True; _ledger.append(1)
assert hasattr(functools, "cache") == True; _ledger.append(1)
assert hasattr(functools, "lru_cache") == True; _ledger.append(1)
assert hasattr(functools, "wraps") == True; _ledger.append(1)
assert hasattr(functools, "update_wrapper") == True; _ledger.append(1)
assert hasattr(functools, "singledispatch") == True; _ledger.append(1)
assert hasattr(functools, "singledispatchmethod") == True; _ledger.append(1)
assert hasattr(functools, "cached_property") == True; _ledger.append(1)
assert hasattr(functools, "cmp_to_key") == True; _ledger.append(1)
assert hasattr(functools, "total_ordering") == True; _ledger.append(1)
assert hasattr(functools, "WRAPPER_ASSIGNMENTS") == True; _ledger.append(1)
assert hasattr(functools, "WRAPPER_UPDATES") == True; _ledger.append(1)

# 6) itertools — hasattr core surface
assert hasattr(itertools, "count") == True; _ledger.append(1)
assert hasattr(itertools, "cycle") == True; _ledger.append(1)
assert hasattr(itertools, "repeat") == True; _ledger.append(1)
assert hasattr(itertools, "chain") == True; _ledger.append(1)
assert hasattr(itertools, "compress") == True; _ledger.append(1)
assert hasattr(itertools, "dropwhile") == True; _ledger.append(1)
assert hasattr(itertools, "takewhile") == True; _ledger.append(1)
assert hasattr(itertools, "islice") == True; _ledger.append(1)
assert hasattr(itertools, "starmap") == True; _ledger.append(1)
assert hasattr(itertools, "filterfalse") == True; _ledger.append(1)
assert hasattr(itertools, "groupby") == True; _ledger.append(1)
assert hasattr(itertools, "accumulate") == True; _ledger.append(1)
assert hasattr(itertools, "tee") == True; _ledger.append(1)
assert hasattr(itertools, "zip_longest") == True; _ledger.append(1)
assert hasattr(itertools, "product") == True; _ledger.append(1)
assert hasattr(itertools, "permutations") == True; _ledger.append(1)
assert hasattr(itertools, "combinations") == True; _ledger.append(1)
assert hasattr(itertools, "combinations_with_replacement") == True; _ledger.append(1)
assert hasattr(itertools, "pairwise") == True; _ledger.append(1)
assert hasattr(itertools, "batched") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_collections_heapq_json_functools_itertools_value_ops {sum(_ledger)} asserts")
