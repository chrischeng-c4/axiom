"""Surface contract for third-party redis package.

# type-regime: monomorphic

Probes: redis.__version__, redis.Redis, redis.ConnectionPool,
redis.from_url, redis.exceptions.
CPython 3.12 is the oracle.
"""

import redis  # type: ignore[import]
import redis.exceptions  # type: ignore[import]

# Core API
assert hasattr(redis, "__version__"), "__version__"
assert hasattr(redis, "Redis"), "Redis"
assert hasattr(redis, "StrictRedis"), "StrictRedis"
assert hasattr(redis, "ConnectionPool"), "ConnectionPool"
assert hasattr(redis, "from_url"), "from_url"
assert hasattr(redis, "exceptions"), "exceptions"

# Version
assert isinstance(redis.__version__, str), \
    f"version type = {type(redis.__version__)!r}"

# Classes are callable
assert callable(redis.Redis), "Redis callable"
assert callable(redis.ConnectionPool), "ConnectionPool callable"
assert callable(redis.from_url), "from_url callable"

# ConnectionPool construction
_pool = redis.ConnectionPool(host="localhost", port=6379, db=0)
assert hasattr(_pool, "connection_class"), "pool.connection_class"
assert hasattr(_pool, "get_connection"), "pool.get_connection"
assert hasattr(_pool, "release"), "pool.release"

# Redis has expected methods
assert hasattr(redis.Redis, "get"), "Redis.get"
assert hasattr(redis.Redis, "set"), "Redis.set"
assert hasattr(redis.Redis, "delete"), "Redis.delete"
assert hasattr(redis.Redis, "exists"), "Redis.exists"
assert hasattr(redis.Redis, "expire"), "Redis.expire"
assert hasattr(redis.Redis, "lpush"), "Redis.lpush"
assert hasattr(redis.Redis, "rpop"), "Redis.rpop"
assert hasattr(redis.Redis, "hset"), "Redis.hset"
assert hasattr(redis.Redis, "hget"), "Redis.hget"
assert hasattr(redis.Redis, "pipeline"), "Redis.pipeline"

# exceptions module
assert hasattr(redis.exceptions, "RedisError"), "RedisError"
assert hasattr(redis.exceptions, "ConnectionError"), "ConnectionError"
assert hasattr(redis.exceptions, "TimeoutError"), "TimeoutError"
assert hasattr(redis.exceptions, "ResponseError"), "ResponseError"
assert issubclass(redis.exceptions.RedisError, Exception), \
    "RedisError < Exception"

# Module attributes stable
_v_ref = redis.__version__
assert redis.__version__ is _v_ref, "__version__ stable"
_r_ref = redis.Redis
assert redis.Redis is _r_ref, "Redis stable"
_cp_ref = redis.ConnectionPool
assert redis.ConnectionPool is _cp_ref, "ConnectionPool stable"
_fu_ref = redis.from_url
assert redis.from_url is _fu_ref, "from_url stable"

print("surface OK")
