# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_isinstance_threading_email_http_ipaddress_silent"
# subject = "cpython321.lang_isinstance_threading_email_http_ipaddress_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_isinstance_threading_email_http_ipaddress_silent.py"
# status = "filled"
# ///
"""cpython321.lang_isinstance_threading_email_http_ipaddress_silent: execute CPython 3.12 seed lang_isinstance_threading_email_http_ipaddress_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `collections.abc.isinstance` / `threading` sync primitives /
# `email` / `http.HTTPStatus` / `http.client` / `ipaddress`
# constructor / `re` flags six-pack pinned to atomic 235:
# `isinstance([], cabc.Iterable)` / `isinstance({}, cabc.Mapping)`
# / `isinstance(set(), cabc.Set)` / `isinstance(iter([]),
# cabc.Iterator)` / `isinstance([], cabc.Sequence)` (the
# documented "built-in container is a virtual subclass of the
# matching abstract base" value contract — mamba's `isinstance`
# silently returns False against every `collections.abc` ABC,
# even though `hasattr(cabc, ...)` confirms the class exists),
# `threading.Lock.acquire / release` / `threading.RLock.
# acquire / release` / `threading.Event.set / clear` /
# `threading.Semaphore.acquire / release` (the documented sync-
# primitive call surface — mamba raises AttributeError at call
# site: "'Lock' object has no attribute 'acquire'", the lock /
# event / semaphore instances are returned as opaque handles
# with no method dispatch), `email.message.Message` /
# `EmailMessage` / `email.utils.formataddr / parseaddr / quote
# / unquote / formatdate / parsedate / getaddresses` /
# `email.message_from_string / message_from_bytes` (the
# documented module-level surface — mamba's `email` /
# `email.message` / `email.utils` module dicts do not expose
# any of them so `hasattr(...)` collapses to False),
# `http.HTTPStatus` / `http.HTTPMethod` (the documented
# top-level enum class surface — mamba's `http` module dict
# does not expose either; the attribute access returns the
# boxed-handle integer 0 / None instead of the enum), `http.
# client.HTTPConnection / HTTPSConnection / HTTPResponse /
# HTTPException / responses` (the documented top-level surface
# — mamba's `http.client` module dict does not expose them),
# `ipaddress.IPv4Address("1.2.3.4")` (the documented "returns
# an IPv4Address instance" value contract — mamba raises
# AttributeError: "'dict' object has no attribute 'IPv4Address'"
# at call site even though `hasattr(ipaddress, "IPv4Address")`
# returns True), and `re.IGNORECASE` / `re.DOTALL` /
# `re.MULTILINE` flag semantics (the documented "flag changes
# match behavior" value contract — mamba's `re.match(...,
# re.IGNORECASE)` silently returns None instead of matching,
# `re.match(..., re.DOTALL)` silently returns None,
# `re.findall(..., re.MULTILINE)` silently returns only the
# first-line match `['a']` instead of `['a', 'b', 'c']`).
#
# Behavioral edges that CONFORM on mamba (collections.abc
# hasattr surface, threading hasattr Condition/Barrier/Timer/
# Thread/current_thread/main_thread/local + active_count/
# get_ident types, re value ops findall/sub/split/escape/
# match.group/search.group/fullmatch.group/compile-match +
# named groups + groups() + re hasattr surface, ipaddress
# hasattr surface, copy/deepcopy basic, string constants,
# bisect bisect_left/right + insort_left/right, heapq full
# surface) are covered in the matching pass fixture
# `test_collections_threading_re_string_bisect_heapq_value_ops`.
from typing import Any
import collections.abc as _cabc_mod
import threading as _threading_mod
import email as _email_mod
import email.message as _email_message_mod
import email.utils as _email_utils_mod
import http as _http_mod
import http.client as _http_client_mod
import ipaddress as _ipaddress_mod
import re as _re_mod

cabc_mod: Any = _cabc_mod
threading_mod: Any = _threading_mod
email_mod: Any = _email_mod
email_message_mod: Any = _email_message_mod
email_utils_mod: Any = _email_utils_mod
http_mod: Any = _http_mod
http_client_mod: Any = _http_client_mod
ipaddress_mod: Any = _ipaddress_mod
re_mod: Any = _re_mod


_ledger: list[int] = []

# 1) collections.abc isinstance — built-in containers are virtual subclasses
#    (mamba: silently returns False against every cabc ABC)
assert isinstance([], cabc_mod.Iterable) == True; _ledger.append(1)
assert isinstance([], cabc_mod.Sequence) == True; _ledger.append(1)
assert isinstance({}, cabc_mod.Mapping) == True; _ledger.append(1)
assert isinstance(set(), cabc_mod.Set) == True; _ledger.append(1)
assert isinstance(iter([]), cabc_mod.Iterator) == True; _ledger.append(1)

# 2) threading.Lock.acquire / release — sync primitive call surface
#    (mamba: AttributeError "'Lock' object has no attribute 'acquire'")
_lk: Any = threading_mod.Lock()
try:
    _r = _lk.acquire()
    _ok = True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _lk.release()
    _ok = True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 3) threading.RLock.acquire / release — same call surface
#    (mamba: AttributeError)
_rlk: Any = threading_mod.RLock()
try:
    _r = _rlk.acquire()
    _ok = True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _rlk.release()
    _ok = True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 4) threading.Event.set / clear — Event call surface
#    (mamba: AttributeError "'Event' object has no attribute 'set'")
_ev: Any = threading_mod.Event()
try:
    _ev.set()
    _ok = True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _ev.clear()
    _ok = True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 5) threading.Semaphore.acquire / release — Semaphore call surface
#    (mamba: AttributeError)
_sem: Any = threading_mod.Semaphore(1)
try:
    _r = _sem.acquire()
    _ok = True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _sem.release()
    _ok = True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 6) email / email.message / email.utils — module-level surface
#    (mamba: module dicts do not expose any of them)
assert hasattr(email_message_mod, "Message") == True; _ledger.append(1)
assert hasattr(email_message_mod, "EmailMessage") == True; _ledger.append(1)
assert hasattr(email_utils_mod, "formataddr") == True; _ledger.append(1)
assert hasattr(email_utils_mod, "parseaddr") == True; _ledger.append(1)
assert hasattr(email_utils_mod, "quote") == True; _ledger.append(1)
assert hasattr(email_utils_mod, "unquote") == True; _ledger.append(1)
assert hasattr(email_utils_mod, "formatdate") == True; _ledger.append(1)
assert hasattr(email_utils_mod, "parsedate") == True; _ledger.append(1)
assert hasattr(email_utils_mod, "getaddresses") == True; _ledger.append(1)
assert hasattr(email_mod, "message_from_string") == True; _ledger.append(1)
assert hasattr(email_mod, "message_from_bytes") == True; _ledger.append(1)

# 7) http.HTTPStatus / HTTPMethod — top-level enum class surface
#    (mamba: module dict does not expose either)
assert hasattr(http_mod, "HTTPStatus") == True; _ledger.append(1)
assert hasattr(http_mod, "HTTPMethod") == True; _ledger.append(1)

# 8) http.client — top-level surface
#    (mamba: module dict does not expose any of them)
assert hasattr(http_client_mod, "HTTPConnection") == True; _ledger.append(1)
assert hasattr(http_client_mod, "HTTPSConnection") == True; _ledger.append(1)
assert hasattr(http_client_mod, "HTTPResponse") == True; _ledger.append(1)
assert hasattr(http_client_mod, "HTTPException") == True; _ledger.append(1)
assert hasattr(http_client_mod, "responses") == True; _ledger.append(1)

# 9) ipaddress.IPv4Address — constructor call surface
#    (mamba: AttributeError at call site even though hasattr is True)
try:
    _addr = ipaddress_mod.IPv4Address("1.2.3.4")
    _ok = str(_addr) == "1.2.3.4"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 10) re.IGNORECASE — flag semantics
#     (mamba: silently returns None instead of matching)
_r_ic = re_mod.match(r"abc", "ABC", re_mod.IGNORECASE)
assert (_r_ic is not None) == True; _ledger.append(1)

# 11) re.DOTALL — flag semantics
#     (mamba: silently returns None)
_r_ds = re_mod.match(r"a.b", "a\nb", re_mod.DOTALL)
assert (_r_ds is not None) == True; _ledger.append(1)

# 12) re.MULTILINE — flag semantics
#     (mamba: silently returns only first-line match)
assert re_mod.findall(r"^\w+", "a\nb\nc", re_mod.MULTILINE) == ["a", "b", "c"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_isinstance_threading_email_http_ipaddress_silent {sum(_ledger)} asserts")
