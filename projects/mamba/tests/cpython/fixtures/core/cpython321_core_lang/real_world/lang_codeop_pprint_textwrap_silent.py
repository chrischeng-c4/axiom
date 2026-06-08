# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_codeop_pprint_textwrap_silent"
# subject = "cpython321.lang_codeop_pprint_textwrap_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_codeop_pprint_textwrap_silent.py"
# status = "filled"
# ///
"""cpython321.lang_codeop_pprint_textwrap_silent: execute CPython 3.12 seed lang_codeop_pprint_textwrap_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(codeop, 'compile_command')`
# (the documented "codeop exposes the compile_command helper" — mamba
# returns False), `hasattr(codeop, 'Compile')` (the documented "codeop
# exposes the Compile class" — mamba returns False), `hasattr(codeop,
# 'CommandCompiler')` (the documented "codeop exposes the
# CommandCompiler class" — mamba returns False), `type(calendar.
# MONDAY).__name__ == 'Day'` (the documented "calendar.MONDAY is an
# IntEnum member of calendar.Day" — mamba returns 'int' — bare int
# instead of Day enum), `type(queue.Queue()).__name__ == 'Queue'` (the
# documented "queue.Queue() constructs a Queue instance" — mamba
# returns 'int' — constructor degrades to int handle), `hasattr(
# pprint, 'PrettyPrinter')` (the documented "pprint exposes the
# PrettyPrinter class" — mamba returns False), `hasattr(pprint, '
# isreadable')` (the documented "pprint exposes the isreadable
# predicate" — mamba returns False), `hasattr(textwrap, 'TextWrapper')
# ` (the documented "textwrap exposes the TextWrapper class" — mamba
# returns False), `textwrap.shorten('Hello world this is long', width=
# 10) == '[...]'` (the documented "shorten collapses a too-long string
# to the placeholder when no token fits" — mamba returns the full
# input — no shortening applied), and `textwrap.fill('abc def ghi',
# width=5) == 'abc\ndef\nghi'` (the documented "fill wraps at the
# requested width and joins with newlines" — mamba returns the
# unwrapped input — no wrapping applied).
# Ten-pack pinned to atomic 306.
#
# Behavioral edges that CONFORM on mamba (calendar — hasattr Calendar/
# TextCalendar/HTMLCalendar/isleap/leapdays/weekday/monthrange/month_
# name/day_name/month_abbr/day_abbr/MONDAY/SUNDAY/timegm + isleap leap
# /non-leap + leapdays + weekday numeric + monthrange tuple + MONDAY/
# SUNDAY ordinals + month_name[1]/day_name[0]/month_abbr[1]/day_abbr[0]
# + len(month_name)/len(day_name) + timegm epoch zero. code — hasattr
# InteractiveInterpreter/InteractiveConsole/compile_command/interact.
# pickletools — hasattr dis/optimize/genops/OpcodeInfo. trace —
# hasattr Trace/CoverageResults. timeit — hasattr timeit/repeat/
# default_timer/Timer + type(timeit.timeit) function. heapq — hasattr
# heappush/heappop/heappushpop/heapify/heapreplace/nlargest/nsmallest/
# merge + heappop returns min + nlargest/nsmallest semantics. bisect —
# hasattr full + bisect_left/bisect_right values. queue — hasattr
# Queue/LifoQueue/PriorityQueue/SimpleQueue/Empty/Full + Queue empty
# True. copy — hasattr copy/deepcopy/Error + copy/deepcopy work.
# pprint — hasattr pprint/pformat. textwrap — hasattr wrap/fill/dedent
# /indent/shorten + dedent value) are covered in the matching pass
# fixture `test_calendar_codeop_pickletools_value_ops`.
import codeop
import calendar
import queue
import pprint
import textwrap


_ledger: list[int] = []

# 1) hasattr(codeop, 'compile_command') — compile_command helper
#    (mamba: returns False)
assert hasattr(codeop, "compile_command") == True; _ledger.append(1)

# 2) hasattr(codeop, 'Compile') — Compile class
#    (mamba: returns False)
assert hasattr(codeop, "Compile") == True; _ledger.append(1)

# 3) hasattr(codeop, 'CommandCompiler') — CommandCompiler class
#    (mamba: returns False)
assert hasattr(codeop, "CommandCompiler") == True; _ledger.append(1)

# 4) type(calendar.MONDAY).__name__ == 'Day' — IntEnum Day member
#    (mamba: returns 'int' — bare int instead of Day enum)
assert type(calendar.MONDAY).__name__ == "Day"; _ledger.append(1)

# 5) type(queue.Queue()).__name__ == 'Queue' — Queue instance
#    (mamba: returns 'int' — constructor degrades to int handle)
assert type(queue.Queue()).__name__ == "Queue"; _ledger.append(1)

# 6) hasattr(pprint, 'PrettyPrinter') — PrettyPrinter class
#    (mamba: returns False)
assert hasattr(pprint, "PrettyPrinter") == True; _ledger.append(1)

# 7) hasattr(pprint, 'isreadable') — isreadable predicate
#    (mamba: returns False)
assert hasattr(pprint, "isreadable") == True; _ledger.append(1)

# 8) hasattr(textwrap, 'TextWrapper') — TextWrapper class
#    (mamba: returns False)
assert hasattr(textwrap, "TextWrapper") == True; _ledger.append(1)

# 9) textwrap.shorten('Hello world this is long', width=10) == '[...]' — collapse placeholder
#    (mamba: returns the full input — no shortening applied)
assert textwrap.shorten("Hello world this is long", width=10) == "[...]"; _ledger.append(1)

# 10) textwrap.fill('abc def ghi', width=5) == 'abc\ndef\nghi' — wrapped output
#     (mamba: returns 'abc def ghi' — no wrapping applied)
assert textwrap.fill("abc def ghi", width=5) == "abc\ndef\nghi"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_codeop_pprint_textwrap_silent {sum(_ledger)} asserts")
