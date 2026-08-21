# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "decimal_context_test__test_asyncio_task_decimal_context"
# subject = "cpython.test_context.DecimalContextTest.test_asyncio_task_decimal_context"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_context.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_context.py::DecimalContextTest::test_asyncio_task_decimal_context
"""Auto-ported test: DecimalContextTest::test_asyncio_task_decimal_context (CPython 3.12 oracle)."""


import asyncio
import decimal
import unittest


def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
async def fractions(t, precision, x, y):
    with decimal.localcontext() as ctx:
        ctx.prec = precision
        a = decimal.Decimal(x) / decimal.Decimal(y)
        await asyncio.sleep(t)
        b = decimal.Decimal(x) / decimal.Decimal(y ** 2)
        return (a, b)

async def main():
    r1, r2 = await asyncio.gather(fractions(0.1, 3, 1, 3), fractions(0.2, 6, 1, 3))
    return (r1, r2)
r1, r2 = asyncio.run(main())

assert str(r1[0]) == '0.333'

assert str(r1[1]) == '0.111'

assert str(r2[0]) == '0.333333'

assert str(r2[1]) == '0.111111'
print("DecimalContextTest::test_asyncio_task_decimal_context: ok")
