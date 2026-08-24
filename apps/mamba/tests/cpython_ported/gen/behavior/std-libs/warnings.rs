use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/warnings/always_captures_every_occurrence.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_always_captures_every_occurrence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "always_captures_every_occurrence"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: simplefilter("always") captures every occurrence: warning the same message 5 times records 5 messages"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    for _i in range(5):
        warnings.warn("repeat", UserWarning)
    assert len(recorded) == 5, f"always captures all: {len(recorded)!r}"

print("always_captures_every_occurrence OK")
"###);
    assert_output(&out, r###"always_captures_every_occurrence OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/catch_warnings_records_multiple.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_catch_warnings_records_multiple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "catch_warnings_records_multiple"
# subject = "warnings.catch_warnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.catch_warnings: catch_warnings(record=True) collects every emitted warning in order; two warns of different categories yield two records whose categories match"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("first", UserWarning)
    warnings.warn("second", DeprecationWarning)
    assert len(recorded) == 2, f"two warnings = {len(recorded)!r}"
    assert issubclass(recorded[0].category, UserWarning), "first is UserWarning"
    assert issubclass(recorded[1].category, DeprecationWarning), "second is DeprecationWarning"

print("catch_warnings_records_multiple OK")
"###);
    assert_output(&out, r###"catch_warnings_records_multiple OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/catch_warnings_records_single.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_catch_warnings_records_single() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "catch_warnings_records_single"
# subject = "warnings.catch_warnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.catch_warnings: catch_warnings(record=True) with simplefilter("always") captures one WarningMessage carrying the category and message text"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("captured warning", UserWarning)
    assert len(recorded) == 1, f"captured = {len(recorded)!r}"
    assert issubclass(recorded[0].category, UserWarning), f"category = {recorded[0].category!r}"
    assert "captured warning" in str(recorded[0].message), f"message = {str(recorded[0].message)!r}"

print("catch_warnings_records_single OK")
"###);
    assert_output(&out, r###"catch_warnings_records_single OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/catch_warnings_restores_filters.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_catch_warnings_restores_filters() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "catch_warnings_restores_filters"
# subject = "warnings.catch_warnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.catch_warnings: catch_warnings restores the filter state on exit: an "error" filter active inside the block is gone afterward, so a later warn no longer raises"""
import warnings

# Inside: an "error" filter is active and warn() raises.
with warnings.catch_warnings():
    warnings.simplefilter("error")
    _raised = False
    try:
        warnings.warn("inside", UserWarning)
    except UserWarning:
        _raised = True
    assert _raised, "error filter active inside block"

# Outside: the error filter is gone, so a fresh always-filter just records.
with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("outside", UserWarning)
    assert len(recorded) == 1, f"normal filter after catch_warnings = {len(recorded)!r}"

print("catch_warnings_restores_filters OK")
"###);
    assert_output(&out, r###"catch_warnings_restores_filters OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/filter_precedence_first_match_wins.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_filter_precedence_first_match_wins() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "filter_precedence_first_match_wins"
# subject = "warnings.filterwarnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.filterwarnings: filters are an ordered list; append=False prepends so a front "ignore" wins over a later "error", suppressing the warning"""
import warnings

with warnings.catch_warnings():
    warnings.resetwarnings()
    warnings.filterwarnings("error", append=True)
    warnings.filterwarnings("ignore", append=False)
    assert warnings.filters[0][0] == "ignore", f"front = {warnings.filters[0][0]!r}"
    # The prepended "ignore" takes precedence, so warn() is suppressed.
    with warnings.catch_warnings(record=True) as recorded:
        # Re-install the same ordering inside the inner snapshot.
        warnings.resetwarnings()
        warnings.filterwarnings("error", append=True)
        warnings.filterwarnings("ignore", append=False)
        warnings.warn("masked by ignore", UserWarning)
        assert len(recorded) == 0, f"ignore wins: {len(recorded)!r}"

print("filter_precedence_first_match_wins OK")
"###);
    assert_output(&out, r###"filter_precedence_first_match_wins OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/filterwarnings_filters_by_category.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_filterwarnings_filters_by_category() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "filterwarnings_filters_by_category"
# subject = "warnings.filterwarnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.filterwarnings: filterwarnings("ignore", category=DeprecationWarning) drops DeprecationWarning while a concurrent UserWarning is still captured"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.filterwarnings("ignore", category=DeprecationWarning)
    warnings.warn("user", UserWarning)
    warnings.warn("deprecated", DeprecationWarning)
    cats = [w.category for w in recorded]
    assert UserWarning in cats, f"UserWarning captured: {cats!r}"
    assert DeprecationWarning not in cats, f"DeprecationWarning ignored: {cats!r}"

print("filterwarnings_filters_by_category OK")
"###);
    assert_output(&out, r###"filterwarnings_filters_by_category OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/filterwarnings_filters_by_message_regex.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_filterwarnings_filters_by_message_regex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "filterwarnings_filters_by_message_regex"
# subject = "warnings.filterwarnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.filterwarnings: filterwarnings("ignore", message=".*skip_me.*") drops messages matching the regex while non-matching messages are kept"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.filterwarnings("ignore", message=".*skip_me.*")
    warnings.warn("skip_me warning", UserWarning)
    warnings.warn("keep_me warning", UserWarning)
    msgs = [str(w.message) for w in recorded]
    assert not any("skip_me" in m for m in msgs), f"skip_me filtered: {msgs!r}"
    assert any("keep_me" in m for m in msgs), f"keep_me kept: {msgs!r}"

print("filterwarnings_filters_by_message_regex OK")
"###);
    assert_output(&out, r###"filterwarnings_filters_by_message_regex OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/formatwarning_appends_source_line.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_formatwarning_appends_source_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "formatwarning_appends_source_line"
# subject = "warnings.formatwarning"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.formatwarning: the optional line= source argument is appended on its own line indented two spaces after the canonical warning line"""
import warnings

line = warnings.formatwarning("old api", DeprecationWarning, "lib.py", 7, line="foo()")
assert line == "lib.py:7: DeprecationWarning: old api\n  foo()\n", f"line = {line!r}"

print("formatwarning_appends_source_line OK")
"###);
    assert_output(&out, r###"formatwarning_appends_source_line OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/formatwarning_renders_canonical_line.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_formatwarning_renders_canonical_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "formatwarning_renders_canonical_line"
# subject = "warnings.formatwarning"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.formatwarning: formatwarning renders '<file>:<lineno>: <Category>: <message>\\n' exactly, e.g. 'app.py:42: UserWarning: disk low\\n'"""
import warnings

line = warnings.formatwarning("disk low", UserWarning, "app.py", 42)
assert line == "app.py:42: UserWarning: disk low\n", f"line = {line!r}"

print("formatwarning_renders_canonical_line OK")
"###);
    assert_output(&out, r###"formatwarning_renders_canonical_line OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/future_warning_category_captured.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_future_warning_category_captured() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "future_warning_category_captured"
# subject = "warnings.warn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn: warn(msg, FutureWarning) records a WarningMessage whose category is a FutureWarning subclass"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("msg", FutureWarning)
    assert issubclass(recorded[0].category, FutureWarning), "FutureWarning captured"

print("future_warning_category_captured OK")
"###);
    assert_output(&out, r###"future_warning_category_captured OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/ignore_filter_suppresses.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_ignore_filter_suppresses() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "ignore_filter_suppresses"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: simplefilter("ignore") suppresses warnings so a record=True catch_warnings collects nothing"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("ignore")
    warnings.warn("ignored", UserWarning)
    assert len(recorded) == 0, f"ignored = {len(recorded)!r}"

print("ignore_filter_suppresses OK")
"###);
    assert_output(&out, r###"ignore_filter_suppresses OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/module_action_emits_once.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_module_action_emits_once() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "module_action_emits_once"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: the "module" action collapses repeats of the same warning from one module to a single emission across 5 warns"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("module")
    for _ in range(5):
        warnings.warn("same module", UserWarning)
    assert len(recorded) == 1, f"module emits once: {len(recorded)!r}"

print("module_action_emits_once OK")
"###);
    assert_output(&out, r###"module_action_emits_once OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/once_action_emits_once.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_once_action_emits_once() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "once_action_emits_once"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: the "once" action collapses identical repeats globally to a single emission across 5 warns"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("once")
    for _ in range(5):
        warnings.warn("just once", UserWarning)
    assert len(recorded) == 1, f"once emits once: {len(recorded)!r}"

print("once_action_emits_once OK")
"###);
    assert_output(&out, r###"once_action_emits_once OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/once_captures_first_occurrence_only.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_once_captures_first_occurrence_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "once_captures_first_occurrence_only"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: simplefilter("once") collapses identical repeats: warning the same message 5 times records exactly 1"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("once")
    for _i in range(5):
        warnings.warn("once_test", UserWarning)
    assert len(recorded) == 1, f"once captures one: {len(recorded)!r}"

print("once_captures_first_occurrence_only OK")
"###);
    assert_output(&out, r###"once_captures_first_occurrence_only OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/resetwarnings_clears_filters.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_resetwarnings_clears_filters() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "resetwarnings_clears_filters"
# subject = "warnings.resetwarnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.resetwarnings: resetwarnings() clears installed filters so a subsequent simplefilter("always") + warn is captured again"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.filterwarnings("ignore")
    warnings.resetwarnings()
    # After reset, the blanket "ignore" is gone; install always and warn.
    warnings.simplefilter("always")
    warnings.warn("after reset", UserWarning)
    assert len(recorded) >= 1, f"after reset, warning captured = {len(recorded)!r}"

print("resetwarnings_clears_filters OK")
"###);
    assert_output(&out, r###"resetwarnings_clears_filters OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/showwarning_override_intercepts.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_showwarning_override_intercepts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "showwarning_override_intercepts"
# subject = "warnings.showwarning"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.showwarning: warn() dispatches through warnings.showwarning, so replacing the hook intercepts every warning's rendered fields; the original hook is restored afterward"""
import warnings

captured = []


def my_show(message, category, filename, lineno, file=None, line=None):
    captured.append((str(message), category.__name__))


original = warnings.showwarning
warnings.showwarning = my_show
try:
    with warnings.catch_warnings():
        warnings.simplefilter("always")
        warnings.warn("hooked", RuntimeWarning)
        warnings.warn("also hooked", FutureWarning)
finally:
    warnings.showwarning = original

assert captured == [("hooked", "RuntimeWarning"), ("also hooked", "FutureWarning")], (
    f"captured = {captured!r}"
)
# Restoring the original hook leaves the default surface intact.
assert callable(warnings.showwarning), "showwarning restored"

print("showwarning_override_intercepts OK")
"###);
    assert_output(&out, r###"showwarning_override_intercepts OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/warn_defaults_to_userwarning.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_warn_defaults_to_userwarning() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warn_defaults_to_userwarning"
# subject = "warnings.warn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn: a bare-string warn() with no category defaults the recorded category to UserWarning"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("no category given")
    assert recorded[0].category is UserWarning, f"default = {recorded[0].category!r}"

print("warn_defaults_to_userwarning OK")
"###);
    assert_output(&out, r###"warn_defaults_to_userwarning OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/warn_explicit_default_emits_once_per_location.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_warn_explicit_default_emits_once_per_location() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warn_explicit_default_emits_once_per_location"
# subject = "warnings.warn_explicit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn_explicit: under the "default" action, repeated warn_explicit at the same (message, category, location) with a shared registry emits only once and populates the registry"""
import warnings

# warn_explicit threads its de-dup bookkeeping through the registry argument.
registry = {}
with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("default")
    for _ in range(4):
        warnings.warn_explicit("repeat", UserWarning, "f.py", 10, registry=registry)
    assert len(recorded) == 1, f"default emits once: {len(recorded)!r}"
    assert registry, "registry was populated"

print("warn_explicit_default_emits_once_per_location OK")
"###);
    assert_output(&out, r###"warn_explicit_default_emits_once_per_location OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/warn_explicit_distinct_locations_each_emit.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_warn_explicit_distinct_locations_each_emit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warn_explicit_distinct_locations_each_emit"
# subject = "warnings.warn_explicit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn_explicit: distinct linenos are distinct registry keys under "default", so warn_explicit at two different locations emits twice"""
import warnings

registry = {}
with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("default")
    warnings.warn_explicit("loc", UserWarning, "f.py", 1, registry=registry)
    warnings.warn_explicit("loc", UserWarning, "f.py", 2, registry=registry)
    assert len(recorded) == 2, f"distinct locations both emit: {len(recorded)!r}"

print("warn_explicit_distinct_locations_each_emit OK")
"###);
    assert_output(&out, r###"warn_explicit_distinct_locations_each_emit OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/warn_instance_infers_category.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_warn_instance_infers_category() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warn_instance_infers_category"
# subject = "warnings.warn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn: warn(DeprecationWarning("...")) infers the category from the instance and keeps the instance as the recorded message object"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn(DeprecationWarning("from instance"))
    assert recorded[0].category is DeprecationWarning, f"inferred = {recorded[0].category!r}"
    assert isinstance(recorded[0].message, DeprecationWarning), "message is the instance"

print("warn_instance_infers_category OK")
"###);
    assert_output(&out, r###"warn_instance_infers_category OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/warn_stacklevel_records_location.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_warn_stacklevel_records_location() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warn_stacklevel_records_location"
# subject = "warnings.warn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn: warn(msg, UserWarning, stacklevel=2) emitted from a helper still records well-typed lineno (int) and filename (str) location fields"""
import warnings


def emit():
    warnings.warn("from caller", UserWarning, stacklevel=2)


with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    emit()
    assert isinstance(recorded[0].lineno, int), f"lineno type = {type(recorded[0].lineno)!r}"
    assert isinstance(recorded[0].filename, str), f"filename type = {type(recorded[0].filename)!r}"

print("warn_stacklevel_records_location OK")
"###);
    assert_output(&out, r###"warn_stacklevel_records_location OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/warnings/warningmessage_attribute_types.py`.
#[test]
fn test_gen_behavior_std_libs_warnings_warningmessage_attribute_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warningmessage_attribute_types"
# subject = "warnings.WarningMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.WarningMessage: a recorded WarningMessage exposes category (Warning subclass), message (text), lineno (int) and filename (str) with the correct types"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("attr check", RuntimeWarning, stacklevel=1)
    msg = recorded[0]
    assert issubclass(msg.category, RuntimeWarning), f"category = {msg.category!r}"
    assert "attr check" in str(msg.message), f"message = {msg.message!r}"
    assert isinstance(msg.lineno, int), f"lineno type = {type(msg.lineno)!r}"
    assert isinstance(msg.filename, str), f"filename type = {type(msg.filename)!r}"

print("warningmessage_attribute_types OK")
"###);
    assert_output(&out, r###"warningmessage_attribute_types OK
"###);
}
