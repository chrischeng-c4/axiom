# yield from delegation — #2801.
#
# Covers PEP 380 `yield from` semantics:
#
#   yield from sub      Delegates iteration AND value-passing to the
#                       sub-iterable. The outer generator suspends on
#                       each sub-yield. When the sub returns, the
#                       VALUE of the `yield from` expression is the
#                       sub's StopIteration.value (i.e. its `return`
#                       value).
#   send into outer     Send values flow through `yield from` to the
#                       sub-generator's `yield` expression.
#   throw into outer    Exception is delivered to the sub's `yield`,
#                       so the sub can handle it.
#
# Clauses:
#   1. Outer yields the sub's yielded sequence verbatim.
#   2. yield from RETURN value of the sub is the value of the
#      `yield from` expression in the outer.
#   3. yield from an iterable (not a generator) iterates it; the
#      expression value is None when the source has no return.
#   4. send() into the outer is delivered to the sub-generator and
#      becomes the value of its yield expression.
#   5. StopIteration.value out of the outer carries the outer's own
#      return value (not the sub's).
#   6. Chained `yield from` — outermost yields the bottom-most's
#      yielded sequence transitively.
#
# Every print line tagged `[yield-from]` so failure output names
# yield-from semantics.


def sub_with_return():
    yield 1
    yield 2
    yield 3
    return "sub-return"


def outer_capture():
    """Captures yield-from expression value as the outer's return."""
    val = yield from sub_with_return()
    return ("outer-done", val)


def outer_simple():
    """Yields the sub's sequence verbatim without inspecting return."""
    yield from sub_with_return()
    yield "outer-trailer"


def outer_iter_source():
    """yield from over a list (no return value)."""
    val = yield from ["a", "b", "c"]
    return ("from-iter", val)


def echo_sub():
    """Subgenerator that echoes whatever is sent in."""
    received = yield "sub-ready"
    while received != "stop":
        received = yield f"sub-echo:{received}"
    return f"sub-done:{received}"


def outer_send_passthrough():
    val = yield from echo_sub()
    return ("outer-done", val)


def deepest():
    yield "d1"
    yield "d2"
    return "deepest-ret"


def middle():
    val = yield from deepest()
    yield "m1"
    return ("middle", val)


def top():
    val = yield from middle()
    return ("top", val)


# Clause 1: outer yields the sub's sequence verbatim.
print("[yield-from] clause-1 verbatim:", list(outer_simple()))


# Clause 2: yield from expression value == sub.return value.
g2 = outer_capture()
yielded = []
try:
    while True:
        yielded.append(next(g2))
except StopIteration as exc:
    return_value = exc.value
print("[yield-from] clause-2 yielded:", yielded)
print("[yield-from] clause-2 return:", return_value)


# Clause 3: yield from over a plain iterable; expression value None.
g3 = outer_iter_source()
yielded3 = []
try:
    while True:
        yielded3.append(next(g3))
except StopIteration as exc:
    return_value3 = exc.value
print("[yield-from] clause-3 yielded:", yielded3)
print("[yield-from] clause-3 return:", return_value3)


# Clause 4: send() passes through to the sub generator.
g4 = outer_send_passthrough()
print("[yield-from] clause-4 ready:", next(g4))
print("[yield-from] clause-4 send-1:", g4.send("hello"))
print("[yield-from] clause-4 send-2:", g4.send("world"))
# Send "stop" — sub returns, then outer captures its value and
# itself returns. The StopIteration.value is the outer's return.
try:
    g4.send("stop")
except StopIteration as exc:
    print("[yield-from] clause-4 outer-return:", exc.value)


# Clause 5: outer's own return value (NOT sub's) shows up as
# StopIteration.value of the outer.
g5 = outer_capture()
outer_return = None
try:
    while True:
        next(g5)
except StopIteration as exc:
    outer_return = exc.value
print("[yield-from] clause-5 outer-return:", outer_return)
# Confirm the outer return is the WRAPPED tuple, not the raw
# sub-return string.
print(
    "[yield-from] clause-5 wraps-sub:",
    outer_return == ("outer-done", "sub-return"),
)


# Clause 6: chained yield from.
g6 = top()
yielded6 = []
try:
    while True:
        yielded6.append(next(g6))
except StopIteration as exc:
    return_value6 = exc.value
print("[yield-from] clause-6 yielded:", yielded6)
print("[yield-from] clause-6 return:", return_value6)
