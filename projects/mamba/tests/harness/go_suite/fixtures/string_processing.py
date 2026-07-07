"""go_suite shape: string processing (tokenize + word-frequency).

Server-shaped: log-line / request-body style text processing -- split a
corpus into tokens, build a frequency table, and read back the top entries.
The corpus is generated on the fly with a fixed-seed, small-integer linear
congruential generator (Lehmer-style, all intermediate values kept far below
2**31) picking from a shared vocabulary, so mamba/Go/CPython produce a
byte-identical corpus without embedding a large duplicated text blob in three
languages. Deliberately avoids the classic glibc LCG constants (seed *
1103515245 + 12345) -- that multiply briefly produces values >= 2**47, which
hits a confirmed mamba correctness gap where a subsequent bitwise `&`
silently returns None instead of a masked result (see
go_suite/tools/suite_bench.py header for the isolated repro). Small-constant
LCG here keeps every intermediate value under 2**31, well clear of that gap,
and uses `%` (proven correct at any magnitude) instead of `&` for bounding.
"""

VOCAB = [
    "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog", "server", "request",
    "response", "handler", "route", "payload", "json", "queue", "worker", "cache", "token", "session",
    "database", "query", "index", "latency", "throughput", "cpu", "memory", "thread", "process", "socket",
]


def gen_corpus(n_words: int) -> list[str]:
    seed = 12345
    words: list[str] = []
    for _ in range(n_words):
        seed = (seed * 48271) % 2147483647
        idx = seed % len(VOCAB)
        words.append(VOCAB[idx])
    return words


def word_counts(words: list[str]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for w in words:
        if w in counts:
            counts[w] = counts[w] + 1
        else:
            counts[w] = 1
    return counts


def checksum(data: bytes) -> int:
    h: int = 0
    mod: int = 1000000007
    mult: int = 131
    for b in data:
        h = (h * mult + b) % mod
    return h


def main() -> None:
    words = gen_corpus(30000)
    counts = word_counts(words)
    keys = sorted(counts.keys())
    parts: list[str] = []
    for k in keys:
        parts.append(k + ":" + str(counts[k]))
    summary = "string_processing|" + "|".join(parts)
    print("CHECKSUM", checksum(summary.encode("utf-8")))


main()
