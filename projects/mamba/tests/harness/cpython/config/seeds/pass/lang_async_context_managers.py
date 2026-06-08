# lang_async_context_managers.py — #3352 axis-1 async context manager (async
# with on user class with __aenter__) seed.
#
# Exercises the async-with surface mamba services today:
#   1. `async with ACM(x) as y:` invokes __aenter__ and binds the return value
#      to `y`
#   2. __aenter__ side effects (setting an attribute on self) are observable
#      after the `async with` block exits
#   3. Nested `async with` blocks compose: inner uses the outer-bound name
#   4. async-with works with a class returning a non-int value (string)
#   5. async-with works with a class returning a list value
#   6. async-with works with no `as` binding (only the side effect)
#
# Mamba quirks (tracked separately):
#   * __aexit__ is NOT invoked on the user class — exit-side cleanup is
#     skipped (a `self.exited = True` flag stays False)
#   * @asynccontextmanager decorator (#3500) — tracked separately
#
# Contract: AssertionError → Fail; MAMBA_ASSERTION_PASS → AssertionPass.
import asyncio

_ledger: list[int] = []

# (1) async with on a class with __aenter__ — binds the returned value
class _ACM:
    def __init__(self, name):
        self.name = name
        self.entered = False
    async def __aenter__(self):
        self.entered = True
        return self.name
    async def __aexit__(self, *args):
        pass

async def _use_acm():
    async with _ACM("hello") as n:
        return n

assert asyncio.run(_use_acm()) == "hello", (
    f"async with binds __aenter__ return value, got {asyncio.run(_use_acm())!r}"
)
_ledger.append(1)

# (2) __aenter__ side effect (sets self.entered = True) is observable after
#     the async-with block exits
_acm = _ACM("x")

async def _enter_then_done():
    async with _acm as _:
        pass

asyncio.run(_enter_then_done())

assert _acm.entered == True, (
    f"__aenter__ side effect (self.entered=True) observable after async-with, "
    f"got entered={_acm.entered!r}"
)
_ledger.append(1)

# (3) Nested `async with` composes: inner sees outer-bound name
class _CM:
    def __init__(self, v):
        self.v = v
    async def __aenter__(self):
        return self.v
    async def __aexit__(self, *args):
        pass

async def _nested():
    async with _CM(10) as a:
        async with _CM(20) as b:
            return a + b

_n = asyncio.run(_nested())
assert _n - 30 == 0, f"nested async with composes 10 + 20 = 30, got {_n!r}"
_ledger.append(1)

# (4) async-with returning a string
class _SCM:
    async def __aenter__(self):
        return "world"
    async def __aexit__(self, *args):
        pass

async def _str_use():
    async with _SCM() as s:
        return s

_s = asyncio.run(_str_use())
assert _s == "world", f"async with returns str 'world', got {_s!r}"
_ledger.append(1)

# (5) async-with returning a list
class _LCM:
    async def __aenter__(self):
        return [1, 2, 3]
    async def __aexit__(self, *args):
        pass

async def _list_use():
    async with _LCM() as lst:
        return lst

_lst = asyncio.run(_list_use())
assert _lst == [1, 2, 3], f"async with returns list, got {_lst!r}"
_ledger.append(1)

# (6) async-with without `as` — only the side effect is observed
class _NCM:
    def __init__(self):
        self.entered = False
    async def __aenter__(self):
        self.entered = True
        return None
    async def __aexit__(self, *args):
        pass

_ncm = _NCM()

async def _no_as_use():
    async with _ncm:
        pass

asyncio.run(_no_as_use())
assert _ncm.entered == True, (
    f"async with without `as` still fires __aenter__, got {_ncm.entered!r}"
)
_ledger.append(1)

# (7) Three-deep nested async with: 1 + 2 + 3 = 6
async def _three_deep():
    async with _CM(1) as a:
        async with _CM(2) as b:
            async with _CM(3) as c:
                return a + b + c

_t = asyncio.run(_three_deep())
assert _t - 6 == 0, f"three-deep async with composes 1+2+3=6, got {_t!r}"
_ledger.append(1)

# (8) Per-instance __aenter__ side effect: two instances of the same class
#     have their own .entered flag
_a1 = _ACM("a1")
_a2 = _ACM("a2")

async def _enter_a1():
    async with _a1 as _: pass

async def _enter_a2():
    async with _a2 as _: pass

asyncio.run(_enter_a1())
asyncio.run(_enter_a2())

assert _a1.entered == True and _a2.entered == True, (
    f"both instances see their own __aenter__ side effect, "
    f"a1={_a1.entered!r} a2={_a2.entered!r}"
)
_ledger.append(1)

# (9) Coroutine value chaining: async with → await inner
class _NumCM:
    def __init__(self, n): self.n = n
    async def __aenter__(self): return self.n
    async def __aexit__(self, *args): pass

async def _double(x): return x * 2

async def _await_inside_acm():
    async with _NumCM(5) as n:
        return await _double(n)

_d = asyncio.run(_await_inside_acm())
assert _d - 10 == 0, (
    f"await inside async-with body works, got {_d!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_async_context_managers {sum(_ledger)} asserts")
