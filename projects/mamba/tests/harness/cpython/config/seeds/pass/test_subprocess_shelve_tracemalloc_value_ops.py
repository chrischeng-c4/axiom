# Operational AssertionPass seed for the value contract of the
# `subprocess` / `shelve` / `tracemalloc` three-pack pinned to
# atomic 205: `subprocess` (the documented full module-level
# helper / class / exception / sentinel identifier hasattr
# surface — `run` / `call` / `check_call` / `check_output` /
# `Popen` / `PIPE` / `STDOUT` / `DEVNULL` /
# `CalledProcessError` / `SubprocessError` / `TimeoutExpired`
# / `CompletedProcess` / `getstatusoutput` / `getoutput` /
# `list2cmdline` + the documented `PIPE == -1` /
# `STDOUT == -2` / `DEVNULL == -3` integer-sentinel value
# contract), `shelve` (the documented full module-level
# class / helper identifier hasattr surface — `Shelf` /
# `BsdDbShelf` / `DbfilenameShelf` / `open`), and
# `tracemalloc` (the documented partial module-level
# helper identifier hasattr surface — `start` / `stop` /
# `is_tracing` / `clear_traces` / `get_traceback_limit` /
# `get_traced_memory` / `reset_peak` / `take_snapshot` +
# the documented `is_tracing()` False -> True after
# `start()` lifecycle contract + the documented
# `get_traced_memory()[0:2]` returning a `(int, int)`
# tuple with non-negative current value).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(gettext, "gettext") / "ngettext" / "dgettext" /
# "dngettext" / "pgettext" / "npgettext" / "find" /
# "translation" / "install" / "textdomain" /
# "bindtextdomain" / "NullTranslations" / "GNUTranslations"
# / "Catalog" all False on mamba, hasattr(tracemalloc,
# "get_tracemalloc_memory") / "Filter" / "DomainFilter" /
# "Frame" / "Snapshot" / "Statistic" / "StatisticDiff" /
# "Trace" / "Traceback" all False on mamba, hasattr(xml.dom,
# "Node") / "DOMException" / "EMPTY_NAMESPACE" /
# "XML_NAMESPACE" / "XMLNS_NAMESPACE" / "XHTML_NAMESPACE" /
# "registerDOMImplementation" / "getDOMImplementation" all
# False on mamba) are covered in the matching spec fixture
# `lang_gettext_tracemalloc_xmldom_silent`.
import subprocess
import shelve
import tracemalloc


_ledger: list[int] = []

# 1) subprocess — full module hasattr surface
assert hasattr(subprocess, "run") == True; _ledger.append(1)
assert hasattr(subprocess, "call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_output") == True; _ledger.append(1)
assert hasattr(subprocess, "Popen") == True; _ledger.append(1)
assert hasattr(subprocess, "PIPE") == True; _ledger.append(1)
assert hasattr(subprocess, "STDOUT") == True; _ledger.append(1)
assert hasattr(subprocess, "DEVNULL") == True; _ledger.append(1)
assert hasattr(subprocess, "CalledProcessError") == True; _ledger.append(1)
assert hasattr(subprocess, "SubprocessError") == True; _ledger.append(1)
assert hasattr(subprocess, "TimeoutExpired") == True; _ledger.append(1)
assert hasattr(subprocess, "CompletedProcess") == True; _ledger.append(1)
assert hasattr(subprocess, "getstatusoutput") == True; _ledger.append(1)
assert hasattr(subprocess, "getoutput") == True; _ledger.append(1)
assert hasattr(subprocess, "list2cmdline") == True; _ledger.append(1)

# 2) subprocess — integer-sentinel value contract
assert subprocess.PIPE == -1; _ledger.append(1)
assert subprocess.STDOUT == -2; _ledger.append(1)
assert subprocess.DEVNULL == -3; _ledger.append(1)

# 3) shelve — full module hasattr surface
assert hasattr(shelve, "Shelf") == True; _ledger.append(1)
assert hasattr(shelve, "BsdDbShelf") == True; _ledger.append(1)
assert hasattr(shelve, "DbfilenameShelf") == True; _ledger.append(1)
assert hasattr(shelve, "open") == True; _ledger.append(1)

# 4) tracemalloc — partial module hasattr surface
#    (get_tracemalloc_memory / Filter / DomainFilter / Frame /
#    Snapshot / Statistic / StatisticDiff / Trace / Traceback
#    all DIVERGE on mamba — moved to spec)
assert hasattr(tracemalloc, "start") == True; _ledger.append(1)
assert hasattr(tracemalloc, "stop") == True; _ledger.append(1)
assert hasattr(tracemalloc, "is_tracing") == True; _ledger.append(1)
assert hasattr(tracemalloc, "clear_traces") == True; _ledger.append(1)
assert hasattr(tracemalloc, "get_traceback_limit") == True; _ledger.append(1)
assert hasattr(tracemalloc, "get_traced_memory") == True; _ledger.append(1)
assert hasattr(tracemalloc, "reset_peak") == True; _ledger.append(1)
assert hasattr(tracemalloc, "take_snapshot") == True; _ledger.append(1)

# 5) tracemalloc — is_tracing lifecycle + get_traced_memory tuple
assert tracemalloc.is_tracing() == False; _ledger.append(1)
tracemalloc.start()
assert tracemalloc.is_tracing() == True; _ledger.append(1)
_cur, _peak = tracemalloc.get_traced_memory()
assert type(_cur).__name__ == "int"; _ledger.append(1)
assert _cur >= 0; _ledger.append(1)
tracemalloc.stop()

# NB: hasattr(gettext, "gettext") / "ngettext" / "dgettext" /
# "dngettext" / "pgettext" / "npgettext" / "find" /
# "translation" / "install" / "textdomain" /
# "bindtextdomain" / "NullTranslations" / "GNUTranslations"
# / "Catalog" all False on mamba, hasattr(tracemalloc,
# "get_tracemalloc_memory") / "Filter" / "DomainFilter" /
# "Frame" / "Snapshot" / "Statistic" / "StatisticDiff" /
# "Trace" / "Traceback" all False on mamba, hasattr(xml.dom,
# "Node") / "DOMException" / "EMPTY_NAMESPACE" /
# "XML_NAMESPACE" / "XMLNS_NAMESPACE" / "XHTML_NAMESPACE" /
# "registerDOMImplementation" / "getDOMImplementation" all
# False on mamba — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_subprocess_shelve_tracemalloc_value_ops {sum(_ledger)} asserts")
