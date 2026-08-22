import asyncio


def cpu_work(seed: int, work: int) -> int:
    return sum((i ^ seed) * 3 for i in range(work))


async def run_round(rep: int) -> list[int]:
    first_work = 80000 if rep % 2 == 0 else 25000
    second_work = 25000 if rep % 2 == 0 else 80000
    gathered = await asyncio.gather(
        asyncio.to_thread(cpu_work, 101, first_work),
        asyncio.to_thread(cpu_work, 211, second_work),
    )
    expected = [
        cpu_work(101, first_work),
        cpu_work(211, second_work),
    ]
    assert gathered == expected
    assert len(gathered) == 2
    assert gathered[0] is not None
    assert gathered[1] is not None
    assert gathered[0] != gathered[1]
    return list(gathered)


async def main() -> None:
    for rep in range(5):
        gathered = await run_round(rep)
        print("ROUND_OK", rep, gathered[0], gathered[1])


if __name__ == "__main__":
    asyncio.run(main())
