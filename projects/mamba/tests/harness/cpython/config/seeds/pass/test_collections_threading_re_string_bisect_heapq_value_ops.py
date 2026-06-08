# Atomic 235 pass conformance — collections.abc / threading / re / ipaddress /
# copy / string / bisect / heapq value ops + hasattr surface that match between
# CPython 3.12 and mamba.
import collections.abc as cabc
import threading
import re
import ipaddress
import copy
import string
import bisect
import heapq


_ledger: list[int] = []

# 1) collections.abc hasattr surface
assert hasattr(cabc, "Iterable") == True; _ledger.append(1)
assert hasattr(cabc, "Iterator") == True; _ledger.append(1)
assert hasattr(cabc, "Sequence") == True; _ledger.append(1)
assert hasattr(cabc, "MutableSequence") == True; _ledger.append(1)
assert hasattr(cabc, "Mapping") == True; _ledger.append(1)
assert hasattr(cabc, "MutableMapping") == True; _ledger.append(1)
assert hasattr(cabc, "Set") == True; _ledger.append(1)
assert hasattr(cabc, "MutableSet") == True; _ledger.append(1)
assert hasattr(cabc, "Hashable") == True; _ledger.append(1)
assert hasattr(cabc, "Sized") == True; _ledger.append(1)
assert hasattr(cabc, "Container") == True; _ledger.append(1)
assert hasattr(cabc, "Callable") == True; _ledger.append(1)
assert hasattr(cabc, "Generator") == True; _ledger.append(1)
assert hasattr(cabc, "Reversible") == True; _ledger.append(1)
assert hasattr(cabc, "Collection") == True; _ledger.append(1)
assert hasattr(cabc, "ItemsView") == True; _ledger.append(1)
assert hasattr(cabc, "KeysView") == True; _ledger.append(1)
assert hasattr(cabc, "ValuesView") == True; _ledger.append(1)

# 2) threading non-sync surface + ident-style value ops
assert hasattr(threading, "Condition") == True; _ledger.append(1)
assert hasattr(threading, "Barrier") == True; _ledger.append(1)
assert hasattr(threading, "Timer") == True; _ledger.append(1)
assert hasattr(threading, "Thread") == True; _ledger.append(1)
assert hasattr(threading, "current_thread") == True; _ledger.append(1)
assert hasattr(threading, "main_thread") == True; _ledger.append(1)
assert hasattr(threading, "local") == True; _ledger.append(1)
assert hasattr(threading, "Lock") == True; _ledger.append(1)
assert hasattr(threading, "RLock") == True; _ledger.append(1)
assert hasattr(threading, "Event") == True; _ledger.append(1)
assert hasattr(threading, "Semaphore") == True; _ledger.append(1)
assert type(threading.active_count()).__name__ == "int"; _ledger.append(1)
assert type(threading.get_ident()).__name__ == "int"; _ledger.append(1)

# 3) re value ops via top-level module funcs (avoiding lambdas — mamba's
#    lambda+chained-.group() combo is separately divergent)
assert re.findall(r"\d+", "a1b22c333") == ["1", "22", "333"]; _ledger.append(1)
assert re.sub(r"\d+", "X", "a1b22c333") == "aXbXcX"; _ledger.append(1)
assert re.split(r"\d+", "a1b22c333") == ["a", "b", "c", ""]; _ledger.append(1)
assert re.escape("a.b*c") == "a\\.b\\*c"; _ledger.append(1)
assert re.findall(r"(\w+)=(\d+)", "a=1 b=2") == [("a", "1"), ("b", "2")]; _ledger.append(1)
_m1 = re.match(r"\d+", "123abc")
assert _m1 is not None and _m1.group() == "123"; _ledger.append(1)
_m2 = re.search(r"\d+", "abc123def")
assert _m2 is not None and _m2.group() == "123"; _ledger.append(1)
_m3 = re.fullmatch(r"\d+", "12345")
assert _m3 is not None and _m3.group() == "12345"; _ledger.append(1)
_m4 = re.compile(r"\d+").match("123")
assert _m4 is not None and _m4.group() == "123"; _ledger.append(1)
_m5 = re.match(r"(\d+)-(\d+)", "12-34")
assert _m5 is not None and _m5.groups() == ("12", "34"); _ledger.append(1)
_m6 = re.match(r"(?P<n>\d+)", "123")
assert _m6 is not None and _m6.group("n") == "123"; _ledger.append(1)

# 4) re hasattr surface
assert hasattr(re, "match") == True; _ledger.append(1)
assert hasattr(re, "search") == True; _ledger.append(1)
assert hasattr(re, "sub") == True; _ledger.append(1)
assert hasattr(re, "split") == True; _ledger.append(1)
assert hasattr(re, "findall") == True; _ledger.append(1)
assert hasattr(re, "fullmatch") == True; _ledger.append(1)
assert hasattr(re, "compile") == True; _ledger.append(1)
assert hasattr(re, "escape") == True; _ledger.append(1)
assert hasattr(re, "Pattern") == True; _ledger.append(1)
assert hasattr(re, "Match") == True; _ledger.append(1)
assert hasattr(re, "IGNORECASE") == True; _ledger.append(1)
assert hasattr(re, "MULTILINE") == True; _ledger.append(1)
assert hasattr(re, "DOTALL") == True; _ledger.append(1)

# 5) ipaddress hasattr surface
assert hasattr(ipaddress, "IPv4Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Interface") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Interface") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_address") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_network") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_interface") == True; _ledger.append(1)

# 6) copy / deepcopy basic value ops
assert copy.copy({}) == {}; _ledger.append(1)
assert copy.copy([1, 2, 3]) == [1, 2, 3]; _ledger.append(1)
assert copy.copy({1, 2, 3}) == {1, 2, 3}; _ledger.append(1)
assert copy.deepcopy({"a": [1, 2, {"b": 3}]}) == {"a": [1, 2, {"b": 3}]}; _ledger.append(1)
assert hasattr(copy, "copy") == True; _ledger.append(1)
assert hasattr(copy, "deepcopy") == True; _ledger.append(1)

# 7) string constants
assert string.ascii_letters[:10] == "abcdefghij"; _ledger.append(1)
assert string.ascii_lowercase[:5] == "abcde"; _ledger.append(1)
assert string.ascii_uppercase[:5] == "ABCDE"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)
assert type(string.punctuation).__name__ == "str"; _ledger.append(1)
assert type(string.whitespace).__name__ == "str"; _ledger.append(1)

# 8) bisect value ops + insort surface
assert bisect.bisect_left([1, 3, 5, 7], 4) == 2; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7], 5) == 3; _ledger.append(1)
assert bisect.bisect([1, 3, 5, 7], 5) == 3; _ledger.append(1)
_xs_a = [1, 3, 5, 7]
bisect.insort_left(_xs_a, 4)
assert _xs_a == [1, 3, 4, 5, 7]; _ledger.append(1)
_xs_b = [1, 3, 5, 7]
bisect.insort_right(_xs_b, 5)
assert _xs_b == [1, 3, 5, 5, 7]; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)

# 9) heapq value ops + full surface
_h = []
for _x in [3, 1, 4, 1, 5]:
    heapq.heappush(_h, _x)
assert heapq.heappop(_h) == 1; _ledger.append(1)
_h2 = [3, 1, 4, 1, 5, 9, 2, 6]
heapq.heapify(_h2)
assert _h2[0] == 1; _ledger.append(1)
assert heapq.nlargest(3, [3, 1, 4, 1, 5, 9, 2, 6]) == [9, 6, 5]; _ledger.append(1)
assert heapq.nsmallest(3, [3, 1, 4, 1, 5, 9, 2, 6]) == [1, 1, 2]; _ledger.append(1)
assert list(heapq.merge([1, 3, 5], [2, 4, 6])) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)
assert heapq.heappushpop([2, 3, 4], 1) == 1; _ledger.append(1)
_h3 = [3, 1, 4]
heapq.heapify(_h3)
assert heapq.heapreplace(_h3, 0) == 1; _ledger.append(1)
assert hasattr(heapq, "heappush") == True; _ledger.append(1)
assert hasattr(heapq, "heappop") == True; _ledger.append(1)
assert hasattr(heapq, "heapify") == True; _ledger.append(1)
assert hasattr(heapq, "nlargest") == True; _ledger.append(1)
assert hasattr(heapq, "nsmallest") == True; _ledger.append(1)
assert hasattr(heapq, "merge") == True; _ledger.append(1)
assert hasattr(heapq, "heappushpop") == True; _ledger.append(1)
assert hasattr(heapq, "heapreplace") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_collections_threading_re_string_bisect_heapq_value_ops {sum(_ledger)} asserts")
