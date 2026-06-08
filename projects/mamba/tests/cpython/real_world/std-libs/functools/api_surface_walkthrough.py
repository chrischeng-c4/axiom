# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "real_world"
# case = "api_surface_walkthrough"
# subject = "functools"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools: a downstream consumer drives reduce, partial, lru_cache/cache, wraps, total_ordering and cached_property together over realistic inputs, asserting each result"""
import functools


# A small "order lines" workload that leans on several functools tools the
# way a real consumer would.
#
# reduce: total an order from its line subtotals.
line_subtotals = [12.5, 4.0, 8.25, 30.25]
total = functools.reduce(lambda acc, x: acc + x, line_subtotals, 0.0)
assert total == 55.0, f"reduce total = {total!r}"


# partial: a currency formatter with the symbol pre-bound.
def _format_money(symbol, amount):
    return f"{symbol}{amount:.2f}"


usd = functools.partial(_format_money, "$")
assert usd(total) == "$55.00", f"partial format = {usd(total)!r}"


# lru_cache: memoize a "tax lookup" so repeated regions reuse the result.
_tax_calls = 0


@functools.lru_cache(maxsize=16)
def _tax_rate(region: str) -> float:
    global _tax_calls
    _tax_calls += 1
    return {"us": 0.07, "eu": 0.20}.get(region, 0.0)


assert _tax_rate("us") == 0.07, "tax us"
assert _tax_rate("us") == 0.07, "tax us cached"
assert _tax_rate("eu") == 0.20, "tax eu"
assert _tax_calls == 2, f"tax body ran {_tax_calls!r} times"
assert _tax_rate.cache_info().hits == 1, "tax cache hit recorded"


# cache: a recursive discount-tier resolver memoized end to end.
@functools.cache
def _tier(n: int) -> int:
    return 0 if n <= 0 else 1 + _tier(n - 5)


assert _tier(12) == 3, f"tier(12) = {_tier(12)!r}"
assert _tier(12) == 3, "tier(12) cached"


# wraps: a logging decorator preserves the wrapped function's identity.
def _audited(fn):
    @functools.wraps(fn)
    def _inner(*args, **kwargs):
        return fn(*args, **kwargs)

    return _inner


@_audited
def _checkout(amount: float) -> float:
    """Apply tax to the order amount."""
    return round(amount * (1 + _tax_rate("us")), 2)


assert _checkout.__name__ == "_checkout", f"wraps name = {_checkout.__name__!r}"
assert _checkout.__doc__ == "Apply tax to the order amount.", "wraps doc"
assert _checkout(100.0) == 107.0, f"checkout = {_checkout(100.0)!r}"


# total_ordering: sortable money-valued records from a single seed op.
@functools.total_ordering
class Order:
    def __init__(self, amount):
        self.amount = amount

    def __eq__(self, other):
        return self.amount == other.amount

    def __lt__(self, other):
        return self.amount < other.amount


orders = [Order(30.0), Order(10.0), Order(20.0)]
ordered = sorted(orders)
assert [o.amount for o in ordered] == [10.0, 20.0, 30.0], "total_ordering sort"
assert Order(10.0) <= Order(10.0), "derived le"
assert Order(30.0) >= Order(20.0), "derived ge"


# cached_property: an expensive per-order computation runs once.
class Invoice:
    def __init__(self, lines):
        self.lines = lines
        self.compute_calls = 0

    @functools.cached_property
    def grand_total(self):
        self.compute_calls += 1
        return functools.reduce(lambda a, x: a + x, self.lines, 0.0)


inv = Invoice(line_subtotals)
assert inv.grand_total == 55.0, f"invoice total = {inv.grand_total!r}"
assert inv.grand_total == 55.0, "invoice total cached"
assert inv.compute_calls == 1, f"invoice computed {inv.compute_calls!r} times"

print("api_surface_walkthrough OK")
