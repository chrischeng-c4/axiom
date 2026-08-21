"""Offline anyio task group dispatch (no sleeps, no sockets).

End-user scenario: async frameworks (httpx, fastapi, starlette,
trio-shimmed libs) all delegate concurrency primitives to anyio.
This fixture exercises the smallest reproducible "anyio runs"
shape: create a task group, start three sleep-free child tasks
that push their results into a shared list, await them all
implicitly via `async with`, and assert deterministic completion.

DoD: this script must exit 0 under both CPython and mamba.

Notes:
- No wall-clock sleeps (`anyio.sleep` is not used). All tasks are
  CPU-bound and complete on the first scheduler tick. This keeps
  the fixture's wall-time independent of host load.
- Result ordering after a task group exits is undefined (any
  scheduler interleaving is legal), so we assert on the
  *set* of collected values, not the list order.
- anyio.run() picks the asyncio backend by default; we pass
  `backend="asyncio"` explicitly so the fixture doesn't depend
  on the optional `trio` backend being installed.
"""

import anyio


async def child(index: int, sink: list[int]) -> None:
    """Push the squared index into the sink without yielding to I/O."""
    sink.append(index * index)


async def main() -> None:
    results: list[int] = []
    async with anyio.create_task_group() as tg:
        for i in range(1, 4):
            tg.start_soon(child, i, results)

    # After the `async with` block exits, every task in the group has
    # joined — anyio's structured-concurrency contract. The sink must
    # carry exactly the three squares.
    assert len(results) == 3, f"task group must run 3 children, got {len(results)}"
    assert set(results) == {1, 4, 9}, (
        f"task group must collect {{1, 4, 9}}, got {sorted(results)}"
    )


anyio.run(main, backend="asyncio")
print("ok: anyio task group joined")
