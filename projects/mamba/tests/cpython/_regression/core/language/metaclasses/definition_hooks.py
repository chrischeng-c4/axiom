"""Metaclass class-definition hooks for class statements and types.new_class."""

import types


events = []


class _Meta(type):
    @classmethod
    def __prepare__(mcls, name, bases):
        events.append("prepare:" + name)
        return {}

    def __new__(mcls, name, bases, namespace):
        events.append("new:" + name)
        namespace["injected"] = name
        return type.__new__(mcls, name, bases, namespace)

    def __init__(cls, name, bases, namespace):
        events.append("init:" + name)
        type.__init__(cls, name, bases, namespace)


class _Statement(metaclass=_Meta):
    body_attr = 1


assert _Statement.injected == "_Statement"
assert _Statement.body_attr == 1
assert events == ["prepare:_Statement", "new:_Statement", "init:_Statement"], events


def _fill_namespace(namespace):
    namespace["body_attr"] = 2


_Dynamic = types.new_class("_Dynamic", (), {"metaclass": _Meta}, _fill_namespace)
assert _Dynamic.injected == "_Dynamic"
assert _Dynamic.body_attr == 2
assert events[-3:] == ["prepare:_Dynamic", "new:_Dynamic", "init:_Dynamic"], events

print("definition_hooks OK")
