# Generator lifecycle — send / throw / close — #2797.
#
# Covers the generator-object lifecycle methods beyond simple
# iteration:
#
#   next(g)          advance to the next yield. Equivalent to
#                    g.send(None).
#   g.send(value)    resume the generator with `value` as the result
#                    of the suspended `yield` expression. First call
#                    on a fresh generator must be next() OR
#                    send(None) — sending a non-None value into an
#                    un-started generator raises TypeError.
#   g.throw(ExcCls)  resume the generator by raising ExcCls AT the
#                    suspended yield point. The generator may handle
#                    it (yields again or returns) or let it
#                    propagate.
#   g.close()        raises GeneratorExit at the suspended yield.
#   StopIteration    raised by next() when the generator returns.
#                    StopIteration.value carries the return value.
#
# Clauses:
#   1. next() drives the generator and yields the expected sequence.
#   2. send(value) injects a value back into the yield expression.
#   3. throw(ExcCls) raises at the yield point; if the generator
#      catches it, throw() returns the next yielded value.
#   4. throw() with an unhandled exception propagates out.
#   5. close() raises GeneratorExit; the generator's finally / except
#      blocks run, but it must not yield another value (otherwise
#      Python raises RuntimeError).
#   6. StopIteration.value is the generator's return value.
#
# Every print line tagged `[generator]` so failure output names
# generator lifecycle semantics.


def echo_until_throw():
    """Yields 'start', then echoes whatever send()s in, and reports
    a thrown exception by yielding 'caught:<msg>'. Returns 'final'
    when exhausted naturally."""
    trace = []
    try:
        received = yield "start"
        while received != "stop":
            try:
                received = yield f"echo:{received}"
            except ValueError as exc:
                # We CATCH the throw and re-enter the loop, yielding
                # a marker that proves we saw the exception.
                trace.append(str(exc))
                received = yield f"caught:{exc}"
    except GeneratorExit:
        # close() path. Do NOT yield — Python would raise
        # RuntimeError if we did.
        trace.append("got-generator-exit")
        # Re-raise per the protocol.
        raise
    return "final"


# Clause 1: next() drives the generator.
g1 = echo_until_throw()
print("[generator] clause-1 first:", next(g1))
print("[generator] clause-1 second:", g1.send("a"))
g1.close()


# Clause 2: send(value) injects into the suspended yield.
g2 = echo_until_throw()
# Initial advance — required because send into un-started gen would
# TypeError unless value is None.
print("[generator] clause-2 initial:", next(g2))
print("[generator] clause-2 send-1:", g2.send("hello"))
print("[generator] clause-2 send-2:", g2.send("world"))
g2.close()


# Clause 2b: send(non-None) into fresh generator raises TypeError.
g2b = echo_until_throw()
try:
    g2b.send("too-early")
    print("[generator] clause-2 fresh-send: <unexpected-no-error>")
except TypeError as exc:
    print("[generator] clause-2 fresh-send:", type(exc).__name__)
g2b.close()


# Clause 3: throw() at yield; generator catches and yields again.
g3 = echo_until_throw()
print("[generator] clause-3 initial:", next(g3))
print("[generator] clause-3 send:", g3.send("first"))
# Now we're suspended at the inner `yield f"echo:..."`. Throw a
# ValueError into it; the generator's except catches and yields.
caught_yield = g3.throw(ValueError("boom"))
print("[generator] clause-3 throw-caught:", caught_yield)
g3.close()


# Clause 4: throw() with an unhandled exception propagates.
def yields_then_unhandled():
    yield 1
    yield 2


g4 = yields_then_unhandled()
next(g4)
try:
    g4.throw(RuntimeError("no-handler"))
    print("[generator] clause-4 throw: <unexpected-no-error>")
except RuntimeError as exc:
    print("[generator] clause-4 throw:", type(exc).__name__, str(exc))


# Clause 5: close() raises GeneratorExit; generator finishes.
g5 = echo_until_throw()
next(g5)
g5.close()
# After close(), next() raises StopIteration.
try:
    next(g5)
    print("[generator] clause-5 next-after-close: <unexpected-no-error>")
except StopIteration:
    print("[generator] clause-5 next-after-close: StopIteration")
# close() on already-closed generator is a no-op (no error).
g5.close()
print("[generator] clause-5 close-idempotent: True")


# Clause 6: StopIteration.value is the generator's return value.
def two_then_return():
    yield 1
    yield 2
    return "ret-value"


g6 = two_then_return()
next(g6)
next(g6)
try:
    next(g6)
    print("[generator] clause-6 return: <unexpected-no-error>")
except StopIteration as exc:
    print("[generator] clause-6 return:", exc.value)
