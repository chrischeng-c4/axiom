# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_select_mmap_multiprocessing_decimal_silent"
# subject = "cpython321.lang_select_mmap_multiprocessing_decimal_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_select_mmap_multiprocessing_decimal_silent.py"
# status = "filled"
# ///
"""cpython321.lang_select_mmap_multiprocessing_decimal_silent: execute CPython 3.12 seed lang_select_mmap_multiprocessing_decimal_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `select` /
# `mmap` / `multiprocessing` / `decimal` / `fractions` /
# `marshal` / `copyreg` / `gc` eight-pack pinned to atomic
# 221: `select` (the documented `hasattr(select, "select")
# / "poll" / "kqueue" / "kevent" / "PIPE_BUF" / "error" /
# "POLLIN" / "POLLOUT" / "POLLERR" / "POLLHUP" / "POLLNVAL"
# == True` extended hasattr surface on this host), `mmap`
# (the documented `hasattr(mmap, "mmap") / "ACCESS_READ" /
# "ACCESS_WRITE" / "ACCESS_COPY" / "ACCESS_DEFAULT" /
# "PROT_READ" / "PROT_WRITE" / "PROT_EXEC" / "MAP_SHARED" /
# "MAP_PRIVATE" / "MAP_ANON" / "MAP_ANONYMOUS" / "PAGESIZE"
# / "ALLOCATIONGRANULARITY" / "error" == True` extended
# hasattr surface), `multiprocessing` (the documented
# `hasattr(multiprocessing, "Pool") / "Pipe" / "Lock" /
# "RLock" / "Semaphore" / "BoundedSemaphore" / "Event" /
# "Condition" / "Barrier" / "Manager" / "Value" / "Array" /
# "active_children" / "get_context" / "get_start_method" /
# "set_start_method" / "freeze_support" / "JoinableQueue" /
# "SimpleQueue" == True` extended hasattr surface),
# `decimal` (the documented `hasattr(decimal, "Context") /
# "DecimalException" / "DivisionByZero" / "InvalidOperation"
# / "Inexact" / "Rounded" / "Subnormal" / "Underflow" /
# "Overflow" / "FloatOperation" / "Clamped" /
# "ConversionSyntax" / "ROUND_HALF_EVEN" / "ROUND_HALF_UP"
# / "ROUND_HALF_DOWN" / "ROUND_UP" / "ROUND_DOWN" /
# "ROUND_CEILING" / "ROUND_FLOOR" / "ROUND_05UP" /
# "BasicContext" / "ExtendedContext" / "DefaultContext" /
# "getcontext" / "setcontext" / "localcontext" / "MAX_PREC"
# / "MAX_EMAX" / "MIN_EMIN" / "HAVE_THREADS" == True`
# extended hasattr surface + the documented
# `type(decimal.Decimal("1.5")).__name__ == "Decimal"`
# constructor value contract), `fractions` (the documented
# `type(fractions.Fraction(3, 4)).__name__ == "Fraction"`
# constructor value contract), `marshal` (the documented
# `type(marshal.dumps([1, 2, 3])).__name__ == "bytes"` and
# `marshal.loads(marshal.dumps([1, 2, 3])) == [1, 2, 3]`
# roundtrip value contract), `copyreg` (the documented
# `hasattr(copyreg, "pickle") / "constructor" /
# "dispatch_table" / "__newobj__" / "__newobj_ex__" /
# "_reconstructor" == True` extended hasattr surface), and
# `gc` (the documented `hasattr(gc, "get_referents") /
# "get_referrers" / "garbage" / "callbacks" == True`
# extended hasattr surface).
#
# Behavioral edges that CONFORM on mamba (signal extended
# hasattr surface, selectors DefaultSelector /
# SelectSelector / PollSelector / KqueueSelector /
# BaseSelector / SelectorKey / EVENT_READ / EVENT_WRITE
# hasattr, multiprocessing Process / Queue /
# current_process / cpu_count hasattr, decimal Decimal /
# fractions Fraction / numbers full / marshal dumps / loads
# / dump / load / version hasattr, weakref full hasattr,
# gc enable / disable / isenabled / collect / get_count /
# get_threshold / set_threshold / get_objects / is_tracked
# / freeze / unfreeze / get_freeze_count / get_stats /
# DEBUG_* hasattr, types full hasattr) are covered in the
# matching pass fixture
# `test_signal_selectors_decimal_fractions_numbers_weakref_gc_types_value_ops`.
from typing import Any
import select as _select_mod
import mmap as _mmap_mod
import multiprocessing as _mp_mod
import decimal as _decimal_mod
import fractions as _fractions_mod
import marshal as _marshal_mod
import copyreg as _copyreg_mod
import gc as _gc_mod

select: Any = _select_mod
mmap: Any = _mmap_mod
mp: Any = _mp_mod
decimal: Any = _decimal_mod
fractions: Any = _fractions_mod
marshal: Any = _marshal_mod
copyreg: Any = _copyreg_mod
gc: Any = _gc_mod


_ledger: list[int] = []

# 1) select — extended module hasattr surface
#    (mamba: every documented host-portable attribute False)
assert hasattr(select, "select") == True; _ledger.append(1)
assert hasattr(select, "poll") == True; _ledger.append(1)
assert hasattr(select, "kqueue") == True; _ledger.append(1)
assert hasattr(select, "kevent") == True; _ledger.append(1)
assert hasattr(select, "PIPE_BUF") == True; _ledger.append(1)
assert hasattr(select, "error") == True; _ledger.append(1)
assert hasattr(select, "POLLIN") == True; _ledger.append(1)
assert hasattr(select, "POLLOUT") == True; _ledger.append(1)
assert hasattr(select, "POLLERR") == True; _ledger.append(1)
assert hasattr(select, "POLLHUP") == True; _ledger.append(1)
assert hasattr(select, "POLLNVAL") == True; _ledger.append(1)

# 2) mmap — extended module hasattr surface
#    (mamba: mmap / ACCESS_* / PROT_* / MAP_* / PAGESIZE /
#    ALLOCATIONGRANULARITY / error all False)
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
assert hasattr(mmap, "error") == True; _ledger.append(1)

# 3) multiprocessing — extended module hasattr surface
#    (mamba: Pool / Pipe / Lock / RLock / Semaphore /
#    BoundedSemaphore / Event / Condition / Barrier /
#    Manager / Value / Array / active_children /
#    get_context / get_start_method / set_start_method /
#    freeze_support / JoinableQueue / SimpleQueue all False)
assert hasattr(mp, "Pool") == True; _ledger.append(1)
assert hasattr(mp, "Pipe") == True; _ledger.append(1)
assert hasattr(mp, "Lock") == True; _ledger.append(1)
assert hasattr(mp, "RLock") == True; _ledger.append(1)
assert hasattr(mp, "Semaphore") == True; _ledger.append(1)
assert hasattr(mp, "BoundedSemaphore") == True; _ledger.append(1)
assert hasattr(mp, "Event") == True; _ledger.append(1)
assert hasattr(mp, "Condition") == True; _ledger.append(1)
assert hasattr(mp, "Barrier") == True; _ledger.append(1)
assert hasattr(mp, "Manager") == True; _ledger.append(1)
assert hasattr(mp, "Value") == True; _ledger.append(1)
assert hasattr(mp, "Array") == True; _ledger.append(1)
assert hasattr(mp, "active_children") == True; _ledger.append(1)
assert hasattr(mp, "get_context") == True; _ledger.append(1)
assert hasattr(mp, "get_start_method") == True; _ledger.append(1)
assert hasattr(mp, "set_start_method") == True; _ledger.append(1)
assert hasattr(mp, "freeze_support") == True; _ledger.append(1)
assert hasattr(mp, "JoinableQueue") == True; _ledger.append(1)
assert hasattr(mp, "SimpleQueue") == True; _ledger.append(1)

# 4) decimal — extended module hasattr surface
#    (mamba: every documented class/constant/context-fn
#    other than Decimal returns False)
assert hasattr(decimal, "Context") == True; _ledger.append(1)
assert hasattr(decimal, "DecimalException") == True; _ledger.append(1)
assert hasattr(decimal, "DivisionByZero") == True; _ledger.append(1)
assert hasattr(decimal, "InvalidOperation") == True; _ledger.append(1)
assert hasattr(decimal, "Inexact") == True; _ledger.append(1)
assert hasattr(decimal, "Rounded") == True; _ledger.append(1)
assert hasattr(decimal, "Subnormal") == True; _ledger.append(1)
assert hasattr(decimal, "Underflow") == True; _ledger.append(1)
assert hasattr(decimal, "Overflow") == True; _ledger.append(1)
assert hasattr(decimal, "FloatOperation") == True; _ledger.append(1)
assert hasattr(decimal, "Clamped") == True; _ledger.append(1)
assert hasattr(decimal, "ConversionSyntax") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_EVEN") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_UP") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_DOWN") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_UP") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_DOWN") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_CEILING") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_FLOOR") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_05UP") == True; _ledger.append(1)
assert hasattr(decimal, "BasicContext") == True; _ledger.append(1)
assert hasattr(decimal, "ExtendedContext") == True; _ledger.append(1)
assert hasattr(decimal, "DefaultContext") == True; _ledger.append(1)
assert hasattr(decimal, "getcontext") == True; _ledger.append(1)
assert hasattr(decimal, "setcontext") == True; _ledger.append(1)
assert hasattr(decimal, "localcontext") == True; _ledger.append(1)
assert hasattr(decimal, "MAX_PREC") == True; _ledger.append(1)
assert hasattr(decimal, "MAX_EMAX") == True; _ledger.append(1)
assert hasattr(decimal, "MIN_EMIN") == True; _ledger.append(1)
assert hasattr(decimal, "HAVE_THREADS") == True; _ledger.append(1)

# 5) decimal — Decimal constructor value contract
#    (mamba: Decimal('1.5') collapses to int handle)
_dec = decimal.Decimal("1.5")
assert type(_dec).__name__ == "Decimal"; _ledger.append(1)

# 6) fractions — Fraction constructor value contract
#    (mamba: Fraction(3, 4) collapses to int handle)
_fr = fractions.Fraction(3, 4)
assert type(_fr).__name__ == "Fraction"; _ledger.append(1)

# 7) marshal — dumps/loads value contract
#    (mamba: dumps returns str instead of bytes,
#    loads returns None instead of original)
_blob = marshal.dumps([1, 2, 3])
assert type(_blob).__name__ == "bytes"; _ledger.append(1)
assert marshal.loads(_blob) == [1, 2, 3]; _ledger.append(1)

# 8) copyreg — extended module hasattr surface
#    (mamba: pickle / constructor / dispatch_table /
#    __newobj__ / __newobj_ex__ / _reconstructor all False)
assert hasattr(copyreg, "pickle") == True; _ledger.append(1)
assert hasattr(copyreg, "constructor") == True; _ledger.append(1)
assert hasattr(copyreg, "dispatch_table") == True; _ledger.append(1)
assert hasattr(copyreg, "__newobj__") == True; _ledger.append(1)
assert hasattr(copyreg, "__newobj_ex__") == True; _ledger.append(1)
assert hasattr(copyreg, "_reconstructor") == True; _ledger.append(1)

# 9) gc — extended module hasattr surface
#    (mamba: get_referents / get_referrers / garbage /
#    callbacks all False)
assert hasattr(gc, "get_referents") == True; _ledger.append(1)
assert hasattr(gc, "get_referrers") == True; _ledger.append(1)
assert hasattr(gc, "garbage") == True; _ledger.append(1)
assert hasattr(gc, "callbacks") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_select_mmap_multiprocessing_decimal_silent {sum(_ledger)} asserts")
