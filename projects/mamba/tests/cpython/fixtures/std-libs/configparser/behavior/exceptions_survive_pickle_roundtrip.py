# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "exceptions_survive_pickle_roundtrip"
# subject = "configparser.Error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.Error: every configparser exception reconstructs identical args/message/repr and named attributes across all pickle protocols, including ParsingError with accumulated per-line errors"""
import configparser
import pickle


def roundtrips(exc, attrs):
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        clone = pickle.loads(pickle.dumps(exc, proto))
        assert clone.args == exc.args, f"args mismatch at proto {proto}"
        assert clone.message == exc.message, f"message mismatch at proto {proto}"
        assert repr(clone) == repr(exc), f"repr mismatch at proto {proto}"
        for name in attrs:
            assert getattr(clone, name) == getattr(exc, name), \
                f"{name} mismatch at proto {proto}"


roundtrips(configparser.Error("value"), [])
roundtrips(configparser.NoSectionError("section"), ["section"])
roundtrips(configparser.NoOptionError("option", "section"), ["option", "section"])
roundtrips(
    configparser.DuplicateSectionError("section", "source", 123),
    ["section", "source", "lineno"],
)
roundtrips(
    configparser.DuplicateOptionError("section", "option", "source", 123),
    ["section", "option", "source", "lineno"],
)
roundtrips(
    configparser.InterpolationError("option", "section", "msg"),
    ["option", "section"],
)
roundtrips(
    configparser.InterpolationMissingOptionError(
        "option", "section", "rawval", "reference"
    ),
    ["option", "section", "reference"],
)
roundtrips(
    configparser.InterpolationDepthError("option", "section", "rawval"),
    ["option", "section"],
)
roundtrips(
    configparser.MissingSectionHeaderError("filename", 123, "line"),
    ["line", "source", "lineno"],
)

# ParsingError accumulates per-line errors that must also survive.
pe = configparser.ParsingError("source")
pe.append(1, "line1")
pe.append(2, "line2")
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    clone = pickle.loads(pickle.dumps(pe, proto))
    assert clone.source == pe.source, f"source mismatch at proto {proto}"
    assert clone.errors == pe.errors, f"errors mismatch at proto {proto}"
    assert repr(clone) == repr(pe), f"repr mismatch at proto {proto}"

print("exceptions_survive_pickle_roundtrip OK")
