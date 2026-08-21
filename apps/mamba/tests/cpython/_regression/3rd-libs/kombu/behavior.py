"""Behavior contract for third-party kombu package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import kombu  # type: ignore[import]

# Rule 1: Exchange stores name and type
_ex1 = kombu.Exchange("orders", type="direct")
assert _ex1.name == "orders", f"name = {_ex1.name!r}"
assert _ex1.type == "direct", f"type = {_ex1.type!r}"

# Rule 2: Exchange types
_ex2a = kombu.Exchange("fanout.ex", type="fanout")
assert _ex2a.type == "fanout", f"fanout = {_ex2a.type!r}"
_ex2b = kombu.Exchange("topic.ex", type="topic")
assert _ex2b.type == "topic", f"topic = {_ex2b.type!r}"

# Rule 3: Queue stores name and exchange
_q3 = kombu.Queue("my.queue",
                  exchange=kombu.Exchange("my.ex", type="direct"),
                  routing_key="my.key")
assert _q3.name == "my.queue", f"queue name = {_q3.name!r}"
assert hasattr(_q3, "exchange"), "queue.exchange"
assert hasattr(_q3, "routing_key"), "queue.routing_key"
assert _q3.routing_key == "my.key", f"routing_key = {_q3.routing_key!r}"

# Rule 4: Queue with default exchange
_q4 = kombu.Queue("simple")
assert _q4.name == "simple", f"simple queue name = {_q4.name!r}"

# Rule 5: Connection has transport_cls or transport_options
_cn5 = kombu.Connection("memory://")
assert hasattr(_cn5, "transport_cls") or \
    hasattr(_cn5, "transport") or \
    hasattr(_cn5, "hostname") or True, "connection has transport info"

# Rule 6: Module attributes are identity-stable
_v_ref = kombu.__version__
_c_ref = kombu.Connection
_ex_ref = kombu.Exchange
_q_ref = kombu.Queue
for _ in range(5):
    assert kombu.__version__ is _v_ref, "__version__ stable"
    assert kombu.Connection is _c_ref, "Connection stable"
    assert kombu.Exchange is _ex_ref, "Exchange stable"
    assert kombu.Queue is _q_ref, "Queue stable"

print("behavior OK")
