"""Type-only UI TD vocabulary; aw parses these names and never executes them."""


class Event:
    pass


class Slot:
    pass


def component(value=None):
    return value


def page(value):
    return value


def token(path: str, value: str, token_type: str):
    pass
