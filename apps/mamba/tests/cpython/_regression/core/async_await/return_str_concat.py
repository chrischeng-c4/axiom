# Regression: async fn returning Str concat result, then awaited + used.
# Previously SIGSEGV'd because mb_coroutine_complete + mb_await did not
# retain the heap result — c.result and the awaiting caller shared the
# same rc=1 ref, so the caller's scope-end release freed the heap object
# and subsequent reads of c.result hit a dangling pointer. Pinning the
# happy path so future async-runtime work doesn't regress it.

async def greet(name):
    return "hello " + name

async def main():
    msg = await greet("world")
    print(msg)
    other = await greet("mamba")
    print(other)
    print(msg)  # second use after a second await — must still be valid

import asyncio
asyncio.run(main())
