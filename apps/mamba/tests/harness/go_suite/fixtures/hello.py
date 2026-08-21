"""go_suite hello shape.

Minimal fixture used ONLY to measure process startup: wall-clock time from
process spawn to the first stdout byte. Not one of the 6 server-shaped
workloads; kept intentionally trivial (no imports beyond nothing, no
allocation) so the recorder's "time to first output" sample isolates
interpreter/runtime startup cost rather than workload cost.

Checksum uses a small modulo-bounded accumulator (h*mult + b) % mod, never a
bitwise mask, and every intermediate value stays far below 2**47 -- see
go_suite/tools/suite_bench.py header comment for why (a real mamba integer
bitwise-AND correctness gap: `x & mask` on values >= 2**47 returns None
instead of a bitwise result; kept out of scope for this suite, tracked
separately). Modulo/arithmetic ops stay correct at any magnitude tested.
"""


def checksum(data: bytes) -> int:
    h: int = 0
    mod: int = 1000000007
    mult: int = 131
    for b in data:
        h = (h * mult + b) % mod
    return h


def main() -> None:
    msg = "hello-go-suite"
    print(msg)
    print("CHECKSUM", checksum(msg.encode("utf-8")))


main()
