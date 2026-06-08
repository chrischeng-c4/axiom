# Benchmark: JSON serialisation and deserialisation.
# Measures: string manipulation, dict/list construction, type conversions.

import json

# Build a complex nested structure and round-trip it through JSON.
data: dict = {
    "users": [
        {"id": i, "name": "user_" + str(i), "score": i * 1.5, "active": i % 2 == 0}
        for i in range(500)
    ],
    "meta": {
        "total": 500,
        "page": 1,
        "tags": ["alpha", "beta", "gamma"] * 10,
    },
}

encoded: str = json.dumps(data)
decoded: dict = json.loads(encoded)
print(len(decoded["users"]))
print(decoded["meta"]["total"])
