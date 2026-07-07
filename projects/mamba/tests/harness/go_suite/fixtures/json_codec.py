"""go_suite shape: JSON encode/decode.

Server-shaped: build typed records, round-trip them through JSON
(marshal -> unmarshal), and fold the decoded fields into a checksum. Typed
throughout (class fields, `list[str]`, `int`) except the JSON payload itself,
which is naturally dict-shaped like a real request/response body.

Deterministic: no randomness, no floats (float formatting differs enough
across json encoders/languages to risk spurious checksum mismatches), no
dict-ordering dependence for the checksum (fields are read back by fixed key
names, not by iterating dict order).
"""

import json


class Record:
    def __init__(self, rec_id: int, name: str, tags: list[str], score: int) -> None:
        self.rec_id: int = rec_id
        self.name: str = name
        self.tags: list[str] = tags
        self.score: int = score


def build_records(n: int) -> list[Record]:
    tag_pool = ["alpha", "beta", "gamma", "delta", "epsilon"]
    out: list[Record] = []
    for i in range(n):
        tags = [tag_pool[i % 5], tag_pool[(i * 3) % 5]]
        out.append(Record(i, "item-" + str(i), tags, (i * 37) % 1000))
    return out


def record_to_obj(r: Record):
    return {"id": r.rec_id, "name": r.name, "tags": r.tags, "score": r.score}


def checksum(data: bytes) -> int:
    h: int = 0
    mod: int = 1000000007
    mult: int = 131
    for b in data:
        h = (h * mult + b) % mod
    return h


def main() -> None:
    records = build_records(500)
    iterations = 30
    total: int = 0
    for _ in range(iterations):
        payload = [record_to_obj(r) for r in records]
        text = json.dumps(payload, sort_keys=True)
        parsed = json.loads(text)
        for obj in parsed:
            total += int(obj["id"])
            total += int(obj["score"])
            total += len(str(obj["name"]))
            for tag in obj["tags"]:
                total += len(str(tag))
        total = total % 1000000007
    summary = "json_codec:" + str(total)
    print("CHECKSUM", checksum(summary.encode("utf-8")))


main()
