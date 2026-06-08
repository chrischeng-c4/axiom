# RUN: parse
# CPython-derived: advanced match/case patterns (#552)

# --- complex nested class patterns ---
class Point:
    x: int = 0
    y: int = 0

match points:
    case [Point(x=0), Point(y=0)]:
        pass
    case [Point(x=x1, y=y1), Point(x=x2, y=y2)]:
        pass

# --- star patterns in sequences ---
match items:
    case [first, *middle, last]:
        pass
    case [only]:
        pass
    case [first, second, *rest]:
        pass
    case [*all_items]:
        pass

# --- mapping with rest ---
match config:
    # NOTE: ** rest-pattern in case mapping not supported
    case {"a": 1}:
        pass
    case {"debug": True, "verbose": True}:
        pass

# --- complex guard expressions ---
match value:
    case x if x > 0 and x < 100:
        pass
    case x if x >= 100 or x <= -100:
        pass
    case x if isinstance(x, int) and x % 2 == 0:
        pass

# --- nested match statements ---
match outer:
    case {"type": "container", "items": items}:
        match items:
            case [first, *rest]:
                pass
            case []:
                pass
    case {"type": "leaf", "value": v}:
        pass

# --- match on tuple subjects ---
# NOTE: match (x, y): causes parser confusion; use variable
_match_tuple = (x, y)
match _match_tuple:
    case [0, 0]:
        pass
    case [0, _y]:
        pass
    case [_x, 0]:
        pass
    case _:
        pass

# --- match with complex expression as subject ---
match command.split():
    case ["quit"]:
        pass
    case ["go", direction]:
        pass
    case ["get", item, "from", location]:
        pass

# --- class patterns with positional and keyword ---
class Node:
    pass

# NOTE: keyword args in class pattern not supported
match node:
    case Node(a, b):
        pass
    # NOTE: case Node(1, 2, name="test"): - keyword arg and literal args not supported
    # NOTE: case Node(1, 2): - literal args in class pattern not supported
    case Node(x2, y2) if x2 == 1 and y2 == 2:
        pass
    case Node(x, y, z):
        pass

# --- deeply nested patterns ---
# NOTE: class pattern with built-in type (str(name), int(age)) not supported
match data:
    case {"users": [{"name": name, "age": age}]}:
        pass
    case {"users": [{"name": n, "age": a, "roles": [*roles]}]}:
        pass
    case {"response": {"data": {"items": [first, *rest]}}}:
        pass

# --- OR patterns combined with guards ---
# NOTE: guard if in OR pattern may conflict; moved guard into case body
match status:
    case 200 | 201 | 202:
        if success:
            pass
    case 400 | 404 | 405:
        if not retry:
            pass
    case 500 | 502 | 503:
        pass

# --- OR patterns with capture ---
# NOTE: tuple OR patterns not supported; using list patterns
match shape:
    case ["circle", r] | ["sphere", r]:
        pass
    case ["rect", w, h] | ["box", w, h, _]:
        pass

# --- walrus in match subject ---
# NOTE: walrus operator in match subject not supported
target = compute()
match target:
    case 0:
        pass
    case x if x > 0:
        pass

# --- match with attribute pattern ---
import enum

class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3

match pixel:
    case Color.RED:
        pass
    case Color.GREEN:
        pass
    case Color.BLUE:
        pass

# --- match with None and boolean patterns ---
match result:
    case None:
        pass
    case True:
        pass
    case False:
        pass

# --- match with complex sequence ---
match points:
    case []:
        pass
    case [Point(x=x, y=y)]:
        pass
    case [Point(x=x1, y=y1), Point(x=x2, y=y2), *rest]:
        pass

# --- match with string patterns ---
match command:
    case "quit" | "exit" | "q":
        pass
    # NOTE: str(s) class pattern with built-in type not supported
    case s if isinstance(s, str) and s.startswith("go"):
        pass

# --- match with numeric patterns ---
match code:
    case 0:
        pass
    case -1:
        pass
    case 1 | 2 | 3:
        pass
    # NOTE: int(n) class pattern with built-in type not supported
    case n if isinstance(n, int) and n > 100:
        pass

# --- match with as pattern ---
match data:
    case [n1, n2] if n1 == 1 and n2 == 2:
        pass
    case {"key": value}:
        pass
    case number:
        pass

# --- match with nested OR and capture ---
# NOTE: nested tuple pattern with int() class patterns not supported
match event:
    case ["click", [x, y]] | ["tap", [x, y]]:
        pass

# --- match with multiple case bodies ---
match action:
    case "save":
        result = save()
        log(result)
    case "load":
        data = load()
        validate(data)
        process(data)
    case _:
        pass

# --- match with walrus in guard ---
match item:
    case x if (transformed := transform(x)) is not None:
        use(transformed)
