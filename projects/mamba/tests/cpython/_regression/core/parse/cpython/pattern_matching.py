# RUN: parse
# CPython-derived: PEP 634 pattern matching (match/case)

# --- literal patterns ---
match command:
    case 0:
        pass
    case 1:
        pass

# --- string literal pattern ---
match status:
    case "ok":
        pass
    case "error":
        pass

# --- boolean and None patterns ---
match value:
    case True:
        pass
    case False:
        pass
    case None:
        pass

# --- negative literal ---
match temp:
    case -1:
        pass
    case 0:
        pass

# --- wildcard pattern ---
match x:
    case _:
        pass

# --- capture pattern ---
match point:
    case x:
        pass

# --- OR pattern ---
match code:
    case 200 | 201 | 204:
        pass
    case 400 | 404:
        pass

# --- sequence pattern ---
match items:
    case []:
        pass
    case [x]:
        pass
    case [x, y]:
        pass
    case [first, *rest]:
        pass

# --- mapping pattern ---
match config:
    case {"debug": True}:
        pass
    case {"mode": mode}:
        pass

# --- constructor pattern ---
match shape:
    case Circle(radius):
        pass
    case Rectangle(w, h):
        pass

# --- class pattern (keyword args) ---
match point:
    case Point(x=0, y=0):
        pass

# --- guard clause ---
match value:
    case x if x > 0:
        pass
    case x if x < 0:
        pass
    case 0:
        pass

# --- dotted path pattern ---
match color:
    case Color.RED:
        pass
    case Color.GREEN:
        pass
