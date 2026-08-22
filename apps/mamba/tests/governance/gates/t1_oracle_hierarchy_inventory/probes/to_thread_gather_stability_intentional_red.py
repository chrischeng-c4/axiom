import asyncio
import sys


def cpu_work(seed: int, work: int) -> int:
    return sum((i ^ seed) * 3 for i in range(work))


async def run_round(rep: int) -> None:
    w0 = 3000 + ((rep + 0) % 8) * 500
    w1 = 3000 + ((rep + 1) % 8) * 500
    w2 = 3000 + ((rep + 2) % 8) * 500
    w3 = 3000 + ((rep + 3) % 8) * 500
    w4 = 3000 + ((rep + 4) % 8) * 500
    w5 = 3000 + ((rep + 5) % 8) * 500
    w6 = 3000 + ((rep + 6) % 8) * 500
    w7 = 3000 + ((rep + 7) % 8) * 500
    gathered = await asyncio.gather(
        asyncio.to_thread(cpu_work, 11, w0),
        asyncio.to_thread(cpu_work, 23, w1),
        asyncio.to_thread(cpu_work, 37, w2),
        asyncio.to_thread(cpu_work, 53, w3),
        asyncio.to_thread(cpu_work, 71, w4),
        asyncio.to_thread(cpu_work, 89, w5),
        asyncio.to_thread(cpu_work, 107, w6),
        asyncio.to_thread(cpu_work, 131, w7),
    )
    expected = [
        cpu_work(11, w0),
        cpu_work(23, w1),
        cpu_work(37, w2),
        cpu_work(53, w3),
        cpu_work(71, w4),
        cpu_work(89, w5),
        cpu_work(107, w6),
        cpu_work(131, w7),
    ]
    assert gathered == expected
    assert len(gathered) == 8
    assert len(set(gathered)) == 8
    for item in gathered:
        assert item is not None


async def main() -> None:
    completed_rounds = 0
    for rep in range(100):
        await run_round(rep)
        completed_rounds += 1

    required_rounds = completed_rounds + 100
    if completed_rounds < required_rounds:
        sys.stderr.write(
            f"MAMBA-T1-FT-GATHER-STABILITY-RED failure: completed {completed_rounds} rounds, required {required_rounds}\n"
        )
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
