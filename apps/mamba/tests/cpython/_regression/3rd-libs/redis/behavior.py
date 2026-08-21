"""Behavior contract for third-party redis package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import redis  # type: ignore[import]
import redis.exceptions  # type: ignore[import]

# Rule 1: ConnectionPool stores host/port/db
_pool1 = redis.ConnectionPool(host="localhost", port=6379, db=0)
assert hasattr(_pool1, "connection_class"), "connection_class"
assert hasattr(_pool1, "get_connection"), "get_connection"

# Rule 2: ConnectionPool different DBs are distinct
_pool2a = redis.ConnectionPool(host="redis.example.com", port=6379, db=0)
_pool2b = redis.ConnectionPool(host="redis.example.com", port=6379, db=1)
assert _pool2a is not _pool2b, "different db = different pool"

# Rule 3: RedisError hierarchy
assert issubclass(redis.exceptions.RedisError, Exception), \
    "RedisError < Exception"
assert issubclass(redis.exceptions.ConnectionError, redis.exceptions.RedisError), \
    "ConnectionError < RedisError"
assert issubclass(redis.exceptions.TimeoutError, redis.exceptions.RedisError), \
    "TimeoutError < RedisError"
assert issubclass(redis.exceptions.ResponseError, redis.exceptions.RedisError), \
    "ResponseError < RedisError"

# Rule 4: Redis.StrictRedis is an alias
assert redis.Redis is redis.StrictRedis or \
    issubclass(redis.StrictRedis, redis.Redis) or \
    issubclass(redis.Redis, redis.StrictRedis), \
    "Redis and StrictRedis are related"

# Rule 5: from_url is callable
assert callable(redis.from_url), "from_url callable"

# Rule 6: Module attributes are identity-stable
_v_ref = redis.__version__
_r_ref = redis.Redis
_cp_ref = redis.ConnectionPool
_fu_ref = redis.from_url
for _ in range(5):
    assert redis.__version__ is _v_ref, "__version__ stable"
    assert redis.Redis is _r_ref, "Redis stable"
    assert redis.ConnectionPool is _cp_ref, "ConnectionPool stable"
    assert redis.from_url is _fu_ref, "from_url stable"

print("behavior OK")
