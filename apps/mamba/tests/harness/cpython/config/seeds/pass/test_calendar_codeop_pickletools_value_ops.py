# Atomic 306 pass conformance — calendar module (hasattr Calendar/Text
# Calendar/HTMLCalendar/isleap/leapdays/weekday/monthrange/month_name/
# day_name/month_abbr/day_abbr/MONDAY/SUNDAY/timegm + value contracts:
# isleap leap/non-leap, leapdays count, weekday numeric, monthrange
# tuple, MONDAY/SUNDAY ordinals, month_name[1]/day_name[0]/month_abbr[1]
# /day_abbr[0] strings, len(month_name)/len(day_name) bounds, timegm
# epoch zero) + code module (hasattr InteractiveInterpreter/Interactive
# Console/compile_command/interact) + pickletools module (hasattr dis/
# optimize/genops/OpcodeInfo) + trace module (hasattr Trace/Coverage
# Results) + timeit module (hasattr timeit/repeat/default_timer/Timer
# + type(timeit.timeit) function) + heapq module (hasattr heappush/
# heappop/heappushpop/heapify/heapreplace/nlargest/nsmallest/merge +
# heappop returns min + nlargest/nsmallest semantics) + bisect module
# (hasattr bisect/bisect_left/bisect_right/insort/insort_left/insort_
# right + bisect_left/bisect_right values) + queue module (hasattr
# Queue/LifoQueue/PriorityQueue/SimpleQueue/Empty/Full + Queue empty
# True) + copy module (hasattr copy/deepcopy/Error + copy list +
# deepcopy nested dict) + pprint module (hasattr pprint/pformat) +
# textwrap module (hasattr wrap/fill/dedent/indent/shorten + dedent
# strips common indent).
# All asserts match between CPython 3.12 and mamba.
import calendar
import code
import pickletools
import trace
import timeit
import heapq
import bisect
import queue
import copy
import pprint
import textwrap


_ledger: list[int] = []

# 1) calendar — hasattr core surface
assert hasattr(calendar, "Calendar") == True; _ledger.append(1)
assert hasattr(calendar, "TextCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "HTMLCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "isleap") == True; _ledger.append(1)
assert hasattr(calendar, "leapdays") == True; _ledger.append(1)
assert hasattr(calendar, "weekday") == True; _ledger.append(1)
assert hasattr(calendar, "monthrange") == True; _ledger.append(1)
assert hasattr(calendar, "month_name") == True; _ledger.append(1)
assert hasattr(calendar, "day_name") == True; _ledger.append(1)
assert hasattr(calendar, "month_abbr") == True; _ledger.append(1)
assert hasattr(calendar, "day_abbr") == True; _ledger.append(1)
assert hasattr(calendar, "MONDAY") == True; _ledger.append(1)
assert hasattr(calendar, "SUNDAY") == True; _ledger.append(1)
assert hasattr(calendar, "timegm") == True; _ledger.append(1)

# 2) calendar — value contracts
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.isleap(2001) == False; _ledger.append(1)
assert calendar.isleap(2024) == True; _ledger.append(1)
assert calendar.leapdays(2000, 2010) == 3; _ledger.append(1)
assert calendar.weekday(2024, 1, 1) == 0; _ledger.append(1)
assert calendar.monthrange(2024, 2) == (3, 29); _ledger.append(1)
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.SUNDAY == 6; _ledger.append(1)
assert calendar.month_name[1] == "January"; _ledger.append(1)
assert calendar.day_name[0] == "Monday"; _ledger.append(1)
assert calendar.month_abbr[1] == "Jan"; _ledger.append(1)
assert calendar.day_abbr[0] == "Mon"; _ledger.append(1)
assert len(calendar.month_name) == 13; _ledger.append(1)
assert len(calendar.day_name) == 7; _ledger.append(1)
assert calendar.timegm((1970, 1, 1, 0, 0, 0, 0, 1, 0)) == 0; _ledger.append(1)

# 3) code — hasattr core surface
assert hasattr(code, "InteractiveInterpreter") == True; _ledger.append(1)
assert hasattr(code, "InteractiveConsole") == True; _ledger.append(1)
assert hasattr(code, "compile_command") == True; _ledger.append(1)
assert hasattr(code, "interact") == True; _ledger.append(1)

# 4) pickletools — hasattr core surface
assert hasattr(pickletools, "dis") == True; _ledger.append(1)
assert hasattr(pickletools, "optimize") == True; _ledger.append(1)
assert hasattr(pickletools, "genops") == True; _ledger.append(1)
assert hasattr(pickletools, "OpcodeInfo") == True; _ledger.append(1)

# 5) trace — hasattr core surface (conformant subset)
assert hasattr(trace, "Trace") == True; _ledger.append(1)
assert hasattr(trace, "CoverageResults") == True; _ledger.append(1)

# 6) timeit — hasattr core surface + type contract
assert hasattr(timeit, "timeit") == True; _ledger.append(1)
assert hasattr(timeit, "repeat") == True; _ledger.append(1)
assert hasattr(timeit, "default_timer") == True; _ledger.append(1)
assert hasattr(timeit, "Timer") == True; _ledger.append(1)
assert type(timeit.timeit).__name__ == "function"; _ledger.append(1)

# 7) heapq — hasattr core surface + value contracts
assert hasattr(heapq, "heappush") == True; _ledger.append(1)
assert hasattr(heapq, "heappop") == True; _ledger.append(1)
assert hasattr(heapq, "heappushpop") == True; _ledger.append(1)
assert hasattr(heapq, "heapify") == True; _ledger.append(1)
assert hasattr(heapq, "heapreplace") == True; _ledger.append(1)
assert hasattr(heapq, "nlargest") == True; _ledger.append(1)
assert hasattr(heapq, "nsmallest") == True; _ledger.append(1)
assert hasattr(heapq, "merge") == True; _ledger.append(1)
_h: list[int] = []
heapq.heappush(_h, 3)
heapq.heappush(_h, 1)
heapq.heappush(_h, 2)
assert heapq.heappop(_h) == 1; _ledger.append(1)
assert heapq.nlargest(3, [1, 2, 3, 4, 5]) == [5, 4, 3]; _ledger.append(1)
assert heapq.nsmallest(3, [5, 4, 3, 2, 1]) == [1, 2, 3]; _ledger.append(1)

# 8) bisect — hasattr core surface + value contracts
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)
assert bisect.bisect_left([1, 2, 3, 4], 3) == 2; _ledger.append(1)
assert bisect.bisect_right([1, 2, 3, 4], 3) == 3; _ledger.append(1)

# 9) queue — hasattr core surface + Queue empty
assert hasattr(queue, "Queue") == True; _ledger.append(1)
assert hasattr(queue, "LifoQueue") == True; _ledger.append(1)
assert hasattr(queue, "PriorityQueue") == True; _ledger.append(1)
assert hasattr(queue, "SimpleQueue") == True; _ledger.append(1)
assert hasattr(queue, "Empty") == True; _ledger.append(1)
assert hasattr(queue, "Full") == True; _ledger.append(1)
assert queue.Queue().empty() == True; _ledger.append(1)

# 10) copy — hasattr core surface + value contracts
assert hasattr(copy, "copy") == True; _ledger.append(1)
assert hasattr(copy, "deepcopy") == True; _ledger.append(1)
assert hasattr(copy, "Error") == True; _ledger.append(1)
assert copy.copy([1, 2, 3]) == [1, 2, 3]; _ledger.append(1)
assert copy.deepcopy({"a": [1]}) == {"a": [1]}; _ledger.append(1)

# 11) pprint — hasattr (conformant subset)
assert hasattr(pprint, "pprint") == True; _ledger.append(1)
assert hasattr(pprint, "pformat") == True; _ledger.append(1)

# 12) textwrap — hasattr core surface (conformant subset) + dedent
assert hasattr(textwrap, "wrap") == True; _ledger.append(1)
assert hasattr(textwrap, "fill") == True; _ledger.append(1)
assert hasattr(textwrap, "dedent") == True; _ledger.append(1)
assert hasattr(textwrap, "indent") == True; _ledger.append(1)
assert hasattr(textwrap, "shorten") == True; _ledger.append(1)
assert textwrap.dedent("  hello\n  world") == "hello\nworld"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_calendar_codeop_pickletools_value_ops {sum(_ledger)} asserts")
