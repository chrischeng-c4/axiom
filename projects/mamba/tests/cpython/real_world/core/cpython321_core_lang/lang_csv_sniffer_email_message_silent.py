# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_csv_sniffer_email_message_silent"
# subject = "cpython321.lang_csv_sniffer_email_message_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_csv_sniffer_email_message_silent.py"
# status = "filled"
# ///
"""cpython321.lang_csv_sniffer_email_message_silent: execute CPython 3.12 seed lang_csv_sniffer_email_message_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences in `csv`
# (Sniffer class identity / instance methods, Dialect / get_dialect
# class identity, field_size_limit being a builtin rather than a python
# lambda), `email.message` (Message / EmailMessage class identity +
# instance construction + payload / header roundtrip), and `selectors`
# (DefaultSelector / SelectSelector / BaseSelector / SelectorKey class
# identity). The matching subset (csv QUOTE_* integer constants +
# selectors EVENT_READ / EVENT_WRITE + dialect-registry surface) is
# covered by `test_csv_quote_constants_dialect_registry_ops`; this
# fixture pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • csv.Sniffer — class with .__name__ == "Sniffer"
#     (mamba: csv.Sniffer is None — hasattr returns False);
#   • csv.Sniffer() — returns a Sniffer instance with .sniff and
#     .has_header instance methods (mamba: AttributeError on call);
#   • csv.Sniffer().sniff(sample) — returns a generated Dialect class
#     (mamba: AttributeError);
#   • csv.Sniffer().has_header(sample) — returns bool
#     (mamba: AttributeError);
#   • csv.Dialect.__name__ == "Dialect" — class identity
#     (mamba: returns None);
#   • csv.get_dialect("excel") — returns a Dialect-typed object
#     (mamba: returns a `csv.excel` stub instance);
#   • csv.field_size_limit — builtin_function_or_method, not a
#     python `function` (mamba: returns a python lambda);
#   • email.message.Message.__name__ == "Message" — class identity
#     (mamba: returns None);
#   • email.message.EmailMessage.__name__ == "EmailMessage"
#     (mamba: returns None);
#   • email.message.Message() — instance with type(_).__name__ ==
#     "Message" (mamba: returns a plain dict);
#   • Message().set_payload / get_payload roundtrip
#     (mamba: AttributeError on Message attribute access);
#   • Message()["Subject"] = "Hi" header roundtrip
#     (mamba: AttributeError);
#   • selectors.DefaultSelector.__name__ — non-empty str
#     (mamba: returns None);
#   • selectors.SelectSelector.__name__ == "SelectSelector"
#     (mamba: returns None);
#   • selectors.BaseSelector.__name__ == "BaseSelector"
#     (mamba: returns None);
#   • selectors.SelectorKey.__name__ == "SelectorKey"
#     (mamba: returns None);
#   • selectors.SelectorKey — class object (mamba: python lambda).
import csv as _csv_mod
import email.message as _em_mod
import selectors as _sel_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing on attribute access — `csv.Sniffer`, `email.message.
# EmailMessage`, and `selectors.SelectorKey` exercise documented
# class-identity contracts that mamba elides at the type-stub level.
csv: Any = _csv_mod
email_message: Any = _em_mod
selectors: Any = _sel_mod

_ledger: list[int] = []

# 1) csv.Sniffer — class identity and instance methods
assert csv.Sniffer.__name__ == "Sniffer"; _ledger.append(1)
assert hasattr(csv, "Sniffer"); _ledger.append(1)
_sn: Any = csv.Sniffer()
assert type(_sn).__name__ == "Sniffer"; _ledger.append(1)
# sniff(sample) returns a generated dialect class
_sample = "a,b,c\n1,2,3\n4,5,6\n"
_d: Any = _sn.sniff(_sample)
assert _d is not None; _ledger.append(1)
# has_header(sample) returns bool
_hh: Any = _sn.has_header("name,age\nAlice,30\nBob,40\n")
assert isinstance(_hh, bool); _ledger.append(1)

# 2) csv.Dialect — class identity
assert csv.Dialect.__name__ == "Dialect"; _ledger.append(1)

# 3) csv.get_dialect("excel") — returns a Dialect-typed object
_xd: Any = csv.get_dialect("excel")
assert type(_xd).__name__ == "Dialect"; _ledger.append(1)

# 4) csv.field_size_limit is a builtin, not a python lambda
assert type(csv.field_size_limit).__name__ == "builtin_function_or_method"; _ledger.append(1)

# 5) email.message.Message — class identity
assert email_message.Message.__name__ == "Message"; _ledger.append(1)
assert email_message.EmailMessage.__name__ == "EmailMessage"; _ledger.append(1)

# 6) Message() — instance construction
_m: Any = email_message.Message()
assert type(_m).__name__ == "Message"; _ledger.append(1)

# 7) Message payload roundtrip
_m.set_payload("hello body")
assert _m.get_payload() == "hello body"; _ledger.append(1)

# 8) Message header roundtrip via __setitem__ / __getitem__
_m2: Any = email_message.Message()
_m2["Subject"] = "Greetings"
assert _m2["Subject"] == "Greetings"; _ledger.append(1)
_m2["From"] = "alice@example.com"
assert _m2["From"] == "alice@example.com"; _ledger.append(1)

# 9) selectors class identity
assert selectors.DefaultSelector.__name__ is not None; _ledger.append(1)
assert isinstance(selectors.DefaultSelector.__name__, str); _ledger.append(1)
assert len(selectors.DefaultSelector.__name__) > 0; _ledger.append(1)
assert selectors.SelectSelector.__name__ == "SelectSelector"; _ledger.append(1)
assert selectors.BaseSelector.__name__ == "BaseSelector"; _ledger.append(1)
assert selectors.SelectorKey.__name__ == "SelectorKey"; _ledger.append(1)

# 10) selectors.SelectorKey is a class (not a python lambda)
assert type(selectors.SelectorKey).__name__ != "function"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_csv_sniffer_email_message_silent {sum(_ledger)} asserts")
