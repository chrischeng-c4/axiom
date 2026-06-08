# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_queue_sched_mmap_silent"
# subject = "cpython321.lang_queue_sched_mmap_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_queue_sched_mmap_silent.py"
# status = "filled"
# ///
"""cpython321.lang_queue_sched_mmap_silent: execute CPython 3.12 seed lang_queue_sched_mmap_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across
# the `queue.Queue` instance class identity contract +
# `sched` module identifier surface + `mmap` module
# constant / class identifier surface pinned by atomic 199:
# `queue.Queue` (the documented `Queue` class identity —
# `type(queue.Queue()).__name__ == "Queue"` on CPython;
# mamba collapses to "int" via the integer-handle pattern),
# `sched` (the documented `scheduler` / `Event` module
# identifier surface), and `mmap` (the documented full
# module-level class / constant identifier surface —
# `mmap` / `ACCESS_READ` / `ACCESS_WRITE` / `ACCESS_COPY` /
# `ACCESS_DEFAULT` / `PROT_READ` / `PROT_WRITE` /
# `PROT_EXEC` / `MAP_SHARED` / `MAP_PRIVATE` / `MAP_ANON`
# / `MAP_ANONYMOUS` / `PAGESIZE` / `ALLOCATIONGRANULARITY`).
#
# The matching subset (full queue hasattr + put / qsize /
# get round-trip values, full heapq hasattr + heapify[0] /
# nlargest / nsmallest values, partial selectors hasattr +
# EVENT_READ / EVENT_WRITE integer values) is covered by
# `test_queue_heapq_selectors_value_ops`; this fixture pins
# the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(queue.Queue()).__name__ == "Queue" — documented
#     class identity (mamba: "int" via integer-handle
#     pattern);
#   • hasattr(sched, "scheduler") is True — documented class
#     identifier (mamba: False);
#   • hasattr(sched, "Event") is True — documented class
#     identifier (mamba: False);
#   • hasattr(mmap, "mmap") is True — documented class
#     identifier (mamba: False);
#   • hasattr(mmap, "ACCESS_READ") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "ACCESS_WRITE") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "ACCESS_COPY") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "ACCESS_DEFAULT") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "PROT_READ") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "PROT_WRITE") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "PROT_EXEC") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "MAP_SHARED") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "MAP_PRIVATE") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "MAP_ANON") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "MAP_ANONYMOUS") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "PAGESIZE") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(mmap, "ALLOCATIONGRANULARITY") is True —
#     documented constant identifier (mamba: False).
import queue as _queue_mod
import sched as _sched_mod
import mmap as _mmap_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identity / module-attribute identifier behavior that
# mamba's bundled type stubs do not surface accurately.
queue: Any = _queue_mod
sched: Any = _sched_mod
mmap: Any = _mmap_mod


_ledger: list[int] = []

# 1) queue.Queue — instance class identity contract
assert type(queue.Queue()).__name__ == "Queue"; _ledger.append(1)

# 2) sched — module identifier surface
assert hasattr(sched, "scheduler") == True; _ledger.append(1)
assert hasattr(sched, "Event") == True; _ledger.append(1)

# 3) mmap — full module class / constant identifier surface
assert hasattr(mmap, "mmap") == True; _ledger.append(1)
assert hasattr(mmap, "ACCESS_READ") == True; _ledger.append(1)
assert hasattr(mmap, "ACCESS_WRITE") == True; _ledger.append(1)
assert hasattr(mmap, "ACCESS_COPY") == True; _ledger.append(1)
assert hasattr(mmap, "ACCESS_DEFAULT") == True; _ledger.append(1)
assert hasattr(mmap, "PROT_READ") == True; _ledger.append(1)
assert hasattr(mmap, "PROT_WRITE") == True; _ledger.append(1)
assert hasattr(mmap, "PROT_EXEC") == True; _ledger.append(1)
assert hasattr(mmap, "MAP_SHARED") == True; _ledger.append(1)
assert hasattr(mmap, "MAP_PRIVATE") == True; _ledger.append(1)
assert hasattr(mmap, "MAP_ANON") == True; _ledger.append(1)
assert hasattr(mmap, "MAP_ANONYMOUS") == True; _ledger.append(1)
assert hasattr(mmap, "PAGESIZE") == True; _ledger.append(1)
assert hasattr(mmap, "ALLOCATIONGRANULARITY") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_queue_sched_mmap_silent {sum(_ledger)} asserts")
