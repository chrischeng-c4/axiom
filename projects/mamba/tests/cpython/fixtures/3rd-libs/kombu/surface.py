"""Surface contract for third-party kombu package.

# type-regime: monomorphic

Probes: kombu.__version__, kombu.Connection, kombu.Exchange,
kombu.Queue, kombu.Consumer, kombu.Producer.
CPython 3.12 is the oracle.
"""

import kombu  # type: ignore[import]

# Core API
assert hasattr(kombu, "__version__"), "__version__"
assert hasattr(kombu, "Connection"), "Connection"
assert hasattr(kombu, "Exchange"), "Exchange"
assert hasattr(kombu, "Queue"), "Queue"
assert hasattr(kombu, "Consumer"), "Consumer"
assert hasattr(kombu, "Producer"), "Producer"
assert hasattr(kombu, "Message"), "Message"

# Version
assert isinstance(kombu.__version__, str), \
    f"version type = {type(kombu.__version__)!r}"

# Classes are callable
assert callable(kombu.Connection), "Connection callable"
assert callable(kombu.Exchange), "Exchange callable"
assert callable(kombu.Queue), "Queue callable"
assert callable(kombu.Consumer), "Consumer callable"
assert callable(kombu.Producer), "Producer callable"

# Exchange construction
_ex = kombu.Exchange("test.exchange", type="direct")
assert hasattr(_ex, "name"), "exchange.name"
assert hasattr(_ex, "type"), "exchange.type"
assert _ex.name == "test.exchange", f"exchange.name = {_ex.name!r}"
assert _ex.type == "direct", f"exchange.type = {_ex.type!r}"

# Queue construction
_q = kombu.Queue("test.queue", exchange=_ex, routing_key="test")
assert hasattr(_q, "name"), "queue.name"
assert hasattr(_q, "exchange"), "queue.exchange"
assert _q.name == "test.queue", f"queue.name = {_q.name!r}"

# Module attributes stable
_v_ref = kombu.__version__
assert kombu.__version__ is _v_ref, "__version__ stable"
_c_ref = kombu.Connection
assert kombu.Connection is _c_ref, "Connection stable"
_ex_ref = kombu.Exchange
assert kombu.Exchange is _ex_ref, "Exchange stable"
_q_ref = kombu.Queue
assert kombu.Queue is _q_ref, "Queue stable"

print("surface OK")
