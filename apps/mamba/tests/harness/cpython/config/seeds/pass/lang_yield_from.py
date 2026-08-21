# lang_yield_from.py - axis-1 PEP 380 yield-from delegation seed (#3356).
#
# Surface (from #3356):
#   1. `yield from subgen` delegates iteration
#   2. Return value from subgen is StopIteration.value (captured via `r = yield from`)
#   3. `send` forwarded through `yield from`
#   4. `throw` forwarded through `yield from`

_ledger: list[int] = []


def values_subgen():
    yield 1
    yield 2
    yield 3
    return "done"


def values_outer():
    r = yield from values_subgen()
    yield r


def send_subgen():
    x = yield 10
    yield x * 2


def send_outer():
    yield from send_subgen()


def throw_subgen():
    try:
        yield 1
    except ValueError:
        yield "subcaught"


def throw_outer():
    yield from throw_subgen()


# 1. `yield from subgen` delegates iteration: outer yields each subgen value in order.
g = values_outer()
assert next(g) == 1, "yield from delegates first subgen yield to outer caller"
_ledger.append(1)

assert next(g) == 2, "yield from delegates middle subgen yield to outer caller"
_ledger.append(1)

assert next(g) == 3, "yield from delegates last subgen yield to outer caller"
_ledger.append(1)

# 2. Return value from subgen is captured into `r = yield from` (StopIteration.value).
assert next(g) == "done", "yield from binds subgen return value into r and outer yields it"
_ledger.append(1)

# 3. `send` forwarded through `yield from` into subgen.
g2 = send_outer()
assert next(g2) == 10, "send_outer first delegated yield is 10"
_ledger.append(1)

assert g2.send(5) == 10, "send(5) reaches subgen: x=5 -> yields x*2 = 10"
_ledger.append(1)

# 4. `throw` forwarded through `yield from` into subgen's except clause.
g3 = throw_outer()
assert next(g3) == 1, "throw_outer first delegated yield is 1"
_ledger.append(1)

assert g3.throw(ValueError) == "subcaught", "throw(ValueError) forwarded through yield from into subgen"
_ledger.append(1)

# 1+2. Multiple subgens chained: yield from inside yield from.
def inner():
    yield "a"
    return "inner-done"

def middle():
    r = yield from inner()
    yield r
    return "middle-done"

def outer_chain():
    r = yield from middle()
    yield r

g4 = outer_chain()
assert next(g4) == "a", "double-nested yield from delegates innermost value"
_ledger.append(1)

assert next(g4) == "inner-done", "innermost return value propagates through middle"
_ledger.append(1)

assert next(g4) == "middle-done", "middle return value propagates through outer"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_yield_from {sum(_ledger)} asserts")
