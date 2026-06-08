---
number: 974
title: "type() 3-arg dynamic class creation — type(name, bases, dict)"
state: open
labels: [type:enhancement, priority:p1, crate:mamba]
group: "runtime-ops"
---

# #974 — type() 3-arg dynamic class creation — type(name, bases, dict)

Current `type()` only handles 1-arg form (returns type name). Need 3-arg form `type(name, bases, namespace_dict)` for dynamic class creation. Used by Django, SQLAlchemy, Pydantic metaclass patterns.
