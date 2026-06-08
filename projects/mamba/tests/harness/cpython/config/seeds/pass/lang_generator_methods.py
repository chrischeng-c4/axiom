# lang_generator_methods.py - axis-1 generator send/throw/close seed (#3354).
#
# Surface (from #3354):
#   1. `gen.send(value)` resumes with value
#   2. `gen.throw(ExceptionType)` raises into generator
#   3. `gen.close()` runs finally + raises GeneratorExit
#   4. Generator exhaustion raises StopIteration
#
# Known mamba quirks (worked around in this seed):
#   - Repeatedly invoking `.close()` on multiple generators in the same
#     module sometimes propagates a spurious GeneratorExit after the
#     first .close(); each `.close()` here runs at most once.

_ledger: list[int] = []


def send_gen():
    x = yield 1
    y = yield x + 10
    return y


def throw_gen():
    try:
        yield 1
    except ValueError:
        yield "caught"


def close_gen(ledger):
    try:
        yield 1
        yield 2
    finally:
        ledger.append("finally-ran")


def exhaust_gen():
    yield 1


# 1. `gen.send(value)` resumes a suspended generator with `value`.
g = send_gen()
assert next(g) == 1, "first yield value is delivered via next()"
_ledger.append(1)

assert g.send(5) == 15, "send(5) resumes generator: x=5 -> yield x+10 -> 15"
_ledger.append(1)

# 4. Generator return / exhaustion via send: StopIteration carries return value.
try:
    g.send(99)
    raise AssertionError("send past final yield should have raised StopIteration")
except StopIteration as si:
    assert si.value == 99, "StopIteration.value carries the generator's return value"
_ledger.append(1)

# 2. `gen.throw(ExceptionType)` raises into the suspended yield.
g2 = throw_gen()
assert next(g2) == 1, "throw_gen yields 1 before throw"
_ledger.append(1)

assert g2.throw(ValueError) == "caught", "throw(ValueError) raised inside generator caught by except"
_ledger.append(1)

# 4. Exhaustion via next(): StopIteration with default value None.
g4 = exhaust_gen()
assert next(g4) == 1, "exhaust_gen yields its single value"
_ledger.append(1)

stop_value_sentinel = object()
stop_value = stop_value_sentinel
try:
    next(g4)
    raise AssertionError("exhausted generator should raise StopIteration on next()")
except StopIteration as si:
    stop_value = si.value
assert stop_value is None, "implicit return at function end yields StopIteration(value=None)"
_ledger.append(1)

# 3. `gen.close()` runs finally block in the generator.
ledger = []
g3 = close_gen(ledger)
assert next(g3) == 1, "close_gen yields 1 before close"
_ledger.append(1)

g3.close()
assert ledger == ["finally-ran"], "close() runs finally block in the generator"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_generator_methods {sum(_ledger)} asserts")
