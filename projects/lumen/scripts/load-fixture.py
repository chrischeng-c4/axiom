#!/usr/bin/env python3
# <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-scripts-load-fixture-py" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
"""Generate a synthetic lumen index fixture.

Emits two files:

* `<output>`         — one JSON document per line (NDJSON). Useful for ad-hoc
                       diffing / re-indexing.
* `<output>.req.json` (where the `.json` suffix on `<output>` is replaced) —
                       a single batched `IndexRequest` body the e2e script
                       feeds straight to `POST /collections/users/index`.

The fixture mixes unique bios (to drive non-trivial search results) with a
small pool of repeated emails (to drive duplicates).
"""

from __future__ import annotations

import argparse
import json
import os
import random
import sys
from pathlib import Path

ADJECTIVES = [
    "fast", "quiet", "fierce", "kind", "bold", "calm", "eager", "proud",
    "happy", "curious", "brave", "gentle", "lucky", "wise", "young", "old",
]
NOUNS = [
    "engineer", "designer", "writer", "doctor", "teacher", "musician",
    "scientist", "explorer", "chef", "pilot", "farmer", "architect",
]
LANGS = [
    "rust", "python", "go", "typescript", "kotlin", "swift", "java",
    "c++", "haskell", "ocaml", "elixir", "ruby",
]
HOBBIES = [
    "hiking", "cycling", "running", "reading", "gaming", "cooking",
    "painting", "writing", "climbing", "skiing", "surfing",
]


def gen_bio(rng: random.Random) -> str:
    adj = rng.choice(ADJECTIVES)
    noun = rng.choice(NOUNS)
    lang = rng.choice(LANGS)
    hobby = rng.choice(HOBBIES)
    return f"a {adj} {noun} who codes in {lang} and loves {hobby}"


def gen_email(rng: random.Random, dup_pool_size: int) -> str:
    # ~5% of records share an email — guarantees the duplicates endpoint
    # has something to return.
    if rng.random() < 0.05:
        idx = rng.randrange(dup_pool_size)
        return f"shared{idx}@example.com"
    return f"user{rng.randrange(10**9)}@example.com"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--count", type=int, default=10_000,
                    help="number of synthetic documents to emit")
    ap.add_argument("--output", required=True,
                    help="path to NDJSON output; the IndexRequest body is "
                         "written to <output>.req.json (or sibling)")
    ap.add_argument("--seed", type=int, default=42,
                    help="RNG seed for reproducibility")
    ap.add_argument("--dup-pool", type=int, default=50,
                    help="size of the shared-email pool (controls how many "
                         "duplicate groups appear)")
    ap.add_argument("--items-per-batch", type=int, default=0,
                    help="if >0, split the IndexItems across multiple request "
                         "bodies of at most this many items each, written as "
                         "<output>.req.000.json, .001.json, … (respects the "
                         "server's bulk-index cap). 0 = one combined body.")
    args = ap.parse_args()

    rng = random.Random(args.seed)

    out_ndjson = Path(args.output)
    # `<output>.req.json` if output ends with .json, else `<output>.req.json`.
    if out_ndjson.suffix == ".json":
        out_req = out_ndjson.with_suffix("").with_suffix(".req.json")
    else:
        out_req = out_ndjson.with_name(out_ndjson.name + ".req.json")

    items: list[dict] = []
    with out_ndjson.open("w", encoding="utf-8") as nd:
        for i in range(args.count):
            external_id = f"u{i:07d}"
            bio = gen_bio(rng)
            email = gen_email(rng, args.dup_pool)
            # Two IndexItems per logical doc: one for `bio`, one for `email`.
            bio_item = {
                "external_id": external_id,
                "field": "bio",
                "value": bio,
            }
            email_item = {
                "external_id": external_id,
                "field": "email",
                "value": email,
            }
            items.append(bio_item)
            items.append(email_item)
            nd.write(json.dumps({"external_id": external_id,
                                 "bio": bio,
                                 "email": email}) + "\n")

    # Split into batches of at most --items-per-batch items (0 = single body).
    # request_id must be unique per batch, else the server dedups later batches
    # as idempotent retries of the first.
    per = args.items_per_batch if args.items_per_batch > 0 else len(items)
    per = max(per, 1)
    batches = [items[i:i + per] for i in range(0, len(items), per)] or [[]]

    written = []
    if args.items_per_batch > 0:
        stem = out_req.with_suffix("")  # drop trailing .json
        for n, chunk in enumerate(batches):
            path = stem.with_name(f"{stem.name}.{n:03d}.json")
            req = {"items": chunk, "request_id": f"e2e-{args.seed}-{args.count}-{n}"}
            with path.open("w", encoding="utf-8") as f:
                json.dump(req, f)
            written.append(path)
    else:
        req = {"items": items, "request_id": f"e2e-{args.seed}-{args.count}"}
        with out_req.open("w", encoding="utf-8") as f:
            json.dump(req, f)
        written.append(out_req)

    total_bytes = sum(os.path.getsize(p) for p in written)
    print(
        f"wrote {args.count} docs ({len(items)} IndexItems) to "
        f"{out_ndjson} + {len(written)} request body file(s) "
        f"({total_bytes} bytes total)",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

# </HANDWRITE>
