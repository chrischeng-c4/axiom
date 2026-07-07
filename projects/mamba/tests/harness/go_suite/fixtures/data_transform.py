"""go_suite shape: data transform pipeline (ETL-style).

Server-shaped: filter -> map -> group-by -> aggregate over a stream of typed
event records, the kind of in-process pipeline a backend does on read-path
fan-in or analytics rollups. Typed throughout: `Event` fields, `dict[int,
list[int]]` group accumulator.
"""


class Event:
    def __init__(self, user_id: int, event_type: str, value: int, ts: int) -> None:
        self.user_id: int = user_id
        self.event_type: str = event_type
        self.value: int = value
        self.ts: int = ts


def build_events(n: int) -> list[Event]:
    types = ["click", "view", "purchase", "refund", "signup"]
    out: list[Event] = []
    for i in range(n):
        out.append(Event(i % 200, types[i % 5], (i * 13) % 500, 1700000000 + i))
    return out


def transform(events: list[Event]) -> dict[int, list[int]]:
    groups: dict[int, list[int]] = {}
    for e in events:
        # drop refunds from the rollup, weight purchases higher -- realistic
        # filter+map step of an ETL pipeline
        if e.event_type == "refund":
            continue
        weight = e.value
        if e.event_type == "purchase":
            weight = weight * 10
        if e.user_id not in groups:
            groups[e.user_id] = []
        groups[e.user_id].append(weight)
    return groups


def checksum(data: bytes) -> int:
    h: int = 0
    mod: int = 1000000007
    mult: int = 131
    for b in data:
        h = (h * mult + b) % mod
    return h


def main() -> None:
    events = build_events(20000)
    groups = transform(events)
    keys = sorted(groups.keys())
    parts: list[str] = []
    for uid in keys:
        vs = groups[uid]
        total = sum(vs)
        cnt = len(vs)
        mx = max(vs)
        mn = min(vs)
        parts.append(str(uid) + ":" + str(total) + ":" + str(cnt) + ":" + str(mx) + ":" + str(mn))
    summary = "data_transform|" + ";".join(parts)
    print("CHECKSUM", checksum(summary.encode("utf-8")))


main()
