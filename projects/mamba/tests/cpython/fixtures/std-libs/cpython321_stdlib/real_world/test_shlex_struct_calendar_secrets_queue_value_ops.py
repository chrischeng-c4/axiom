# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_shlex_struct_calendar_secrets_queue_value_ops"
# subject = "cpython321.test_shlex_struct_calendar_secrets_queue_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_shlex_struct_calendar_secrets_queue_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_shlex_struct_calendar_secrets_queue_value_ops: execute CPython 3.12 seed test_shlex_struct_calendar_secrets_queue_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of five
# bootstrap stdlib modules used by every shell-quoting / binary-
# packing / calendar-arithmetic / cryptographic-token / FIFO-queue
# path: `shlex` (the documented `split` / `quote` / `join` shell-
# lexer surface), `struct` (the documented `pack` / `unpack` /
# `calcsize` binary-format helpers), `calendar` (the documented
# `isleap` / `month_name` / `day_name` / `monthrange` / `weekday`
# / `MONDAY` / `SUNDAY` calendar-arithmetic surface), `secrets`
# (the documented `token_bytes` / `token_hex` / `token_urlsafe` /
# `choice` / `randbelow` / `compare_digest` / `SystemRandom`
# attribute surface), and `queue` (the documented `Queue` /
# `LifoQueue` / `PriorityQueue` / `Empty` class identifiers +
# single-thread put / get / qsize / empty contract on `Queue`).
#
# The matching subset between mamba and CPython is the shlex
# split / quote / join layer, struct pack / unpack / calcsize
# layer, calendar isleap + month_name / day_name index + monthrange
# tuple + weekday layer, secrets attribute-surface layer +
# token_bytes / token_hex output type+length layer + choice /
# randbelow validity layer, queue attribute-surface layer +
# single-thread Queue put / get_nowait / qsize / empty layer.
#
# Surface in this fixture:
#   • shlex — split / quote / join chain on string literals;
#   • struct — pack / unpack / calcsize on big-endian int and
#     multi-int + float typecodes;
#   • calendar — isleap / month_name / day_name / monthrange /
#     weekday + module attribute surface;
#   • secrets — token_bytes / token_hex / token_urlsafe /
#     choice / randbelow / compare_digest / SystemRandom
#     attribute surface + output type+length contracts;
#   • queue — single-thread Queue put / get_nowait / qsize /
#     empty chain + LifoQueue / PriorityQueue / Empty
#     attribute surface.
#
# Behavioral edges that DIVERGE on mamba (textwrap.wrap returns
# the unsplit input as a single-element list, textwrap.fill
# returns the unsplit input with no newlines, textwrap.shorten
# returns the unshortened input, textwrap.indent strips the
# trailing newline, html.escape(quote=False) still escapes
# single-quotes, html.unescape on numeric entities returns the
# raw `&#NN;` literal) are covered in the matching spec fixture
# `lang_textwrap_html_silent`.
import shlex
import struct
import calendar
import secrets
import queue


_ledger: list[int] = []

# 1) shlex — split / quote / join
assert shlex.split("hello world") == ["hello", "world"]; _ledger.append(1)
assert shlex.split('"foo bar" baz') == ["foo bar", "baz"]; _ledger.append(1)
assert shlex.quote("a b c") == "'a b c'"; _ledger.append(1)
assert shlex.join(["a", "b c", "d"]) == "a 'b c' d"; _ledger.append(1)
assert hasattr(shlex, "split") == True; _ledger.append(1)
assert hasattr(shlex, "quote") == True; _ledger.append(1)
assert hasattr(shlex, "join") == True; _ledger.append(1)

# 2) struct — pack / unpack / calcsize
assert struct.pack(">i", 42) == b"\x00\x00\x00\x2a"; _ledger.append(1)
assert struct.unpack(">i", b"\x00\x00\x00\x2a") == (42,); _ledger.append(1)
assert struct.calcsize(">i") == 4; _ledger.append(1)
assert struct.calcsize(">ii") == 8; _ledger.append(1)
assert struct.pack(">2i", 1, 2) == b"\x00\x00\x00\x01\x00\x00\x00\x02"; _ledger.append(1)
assert struct.unpack(">2i", b"\x00\x00\x00\x01\x00\x00\x00\x02") == (1, 2); _ledger.append(1)
assert len(struct.pack(">f", 3.14)) == 4; _ledger.append(1)
assert hasattr(struct, "pack") == True; _ledger.append(1)
assert hasattr(struct, "unpack") == True; _ledger.append(1)
assert hasattr(struct, "calcsize") == True; _ledger.append(1)

# 3) calendar — isleap / month_name / day_name / monthrange /
#    weekday + module attribute surface
assert calendar.isleap(2024) == True; _ledger.append(1)
assert calendar.isleap(2023) == False; _ledger.append(1)
assert calendar.isleap(1900) == False; _ledger.append(1)
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.month_name[1] == "January"; _ledger.append(1)
assert calendar.month_name[12] == "December"; _ledger.append(1)
assert calendar.day_name[0] == "Monday"; _ledger.append(1)
assert calendar.monthrange(2024, 2) == (3, 29); _ledger.append(1)
assert calendar.monthrange(2023, 2) == (2, 28); _ledger.append(1)
assert calendar.weekday(2024, 1, 1) == 0; _ledger.append(1)
assert hasattr(calendar, "isleap") == True; _ledger.append(1)
assert hasattr(calendar, "monthrange") == True; _ledger.append(1)
assert hasattr(calendar, "month_name") == True; _ledger.append(1)
assert hasattr(calendar, "day_name") == True; _ledger.append(1)

# 4) secrets — module attribute surface + output type contracts
assert hasattr(secrets, "token_bytes") == True; _ledger.append(1)
assert hasattr(secrets, "token_hex") == True; _ledger.append(1)
assert hasattr(secrets, "token_urlsafe") == True; _ledger.append(1)
assert hasattr(secrets, "choice") == True; _ledger.append(1)
assert hasattr(secrets, "randbelow") == True; _ledger.append(1)
assert hasattr(secrets, "compare_digest") == True; _ledger.append(1)
assert hasattr(secrets, "SystemRandom") == True; _ledger.append(1)
_tb = secrets.token_bytes(8)
assert isinstance(_tb, bytes); _ledger.append(1)
assert len(_tb) == 8; _ledger.append(1)
_th = secrets.token_hex(8)
assert isinstance(_th, str); _ledger.append(1)
assert len(_th) == 16; _ledger.append(1)
_tu = secrets.token_urlsafe(8)
assert isinstance(_tu, str); _ledger.append(1)
_rb = secrets.randbelow(10)
assert 0 <= _rb < 10; _ledger.append(1)
assert secrets.choice([1, 2, 3, 4, 5]) in [1, 2, 3, 4, 5]; _ledger.append(1)
assert secrets.compare_digest("abc", "abc") == True; _ledger.append(1)
assert secrets.compare_digest("abc", "xyz") == False; _ledger.append(1)

# 5) queue — module attribute + single-thread Queue chain
assert hasattr(queue, "Queue") == True; _ledger.append(1)
assert hasattr(queue, "LifoQueue") == True; _ledger.append(1)
assert hasattr(queue, "PriorityQueue") == True; _ledger.append(1)
assert hasattr(queue, "Empty") == True; _ledger.append(1)
_q: queue.Queue = queue.Queue()
_q.put(1)
_q.put(2)
_q.put(3)
assert _q.qsize() == 3; _ledger.append(1)
assert _q.get_nowait() == 1; _ledger.append(1)
assert _q.get_nowait() == 2; _ledger.append(1)
assert _q.empty() == False; _ledger.append(1)
assert _q.get_nowait() == 3; _ledger.append(1)
assert _q.empty() == True; _ledger.append(1)

# NB: textwrap.wrap returns single-element list (no width
# enforcement), textwrap.fill returns no newlines,
# textwrap.shorten no-op, textwrap.indent strips trailing
# newline, html.escape(quote=False) still escapes single-quotes,
# html.unescape on numeric entities (&#NN;) returns the raw
# literal — all DIVERGE on mamba — moved to the divergence-spec
# fixture.

print(f"MAMBA_ASSERTION_PASS: test_shlex_struct_calendar_secrets_queue_value_ops {sum(_ledger)} asserts")
