# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "real_world"
# case = "typed_record_pipeline"
# subject = "typing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: a downstream consumer drives the typing runtime surface together end to end: a NamedTuple record type and a TypedDict view, a runtime_checkable Protocol gate on real instances, cast/assert_type identity pass-through, and get_origin/get_args introspection over Optional/Union/List annotations, asserting a deterministic aggregate over the processed records"""
from typing import (
    List, NamedTuple, Optional, Protocol, TypedDict, Union, assert_type, cast,
    get_args, get_origin, runtime_checkable,
)


# A typed record domain: a NamedTuple value type and a TypedDict view of it.
class Order(NamedTuple):
    id: int
    amount: int
    note: Optional[str] = None


class OrderView(TypedDict):
    id: int
    amount: int


# A structural gate over anything that exposes the order's fields.
@runtime_checkable
class HasAmount(Protocol):
    amount: int


orders = [
    Order(1, 100),
    Order(2, 250, "rush"),
    Order(3, 50),
    Order(4, 600, "bulk"),
]

# Every order is structurally a HasAmount; a bare int is not.
for o in orders:
    assert isinstance(o, HasAmount) is True
assert isinstance(99, HasAmount) is False

# cast / assert_type are runtime identity pass-throughs in the pipeline.
total = 0
views: List[OrderView] = []
for o in orders:
    amount = cast(int, o.amount)          # no coercion: stays the same int
    assert assert_type(o, Order) is o     # identity, never a copy
    total += amount
    views.append({"id": o.id, "amount": amount})

assert total == 1000
assert len(views) == 4
assert views[1] == {"id": 2, "amount": 250}

# Introspect the record's declared annotations via get_origin / get_args.
ann = Order.__annotations__
assert get_origin(ann["note"]) is Union               # Optional[str] is a Union
assert get_args(ann["note"]) == (str, type(None))
assert get_origin(List[OrderView]) is list
assert get_args(List[OrderView]) == (OrderView,)
assert get_origin(int) is None                        # a bare type has no origin

# A deterministic aggregate over the processed records.
assert sum(v["amount"] for v in views) == total

print("typed_record_pipeline OK")
