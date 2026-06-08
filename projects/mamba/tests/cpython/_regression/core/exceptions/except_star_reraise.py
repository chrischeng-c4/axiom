# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# except* propagates unmatched exception subgroups

def run(group):
    try:
        raise group
    except* ValueError as eg:
        print("caught ValueError group, size", len(eg.exceptions))

# Group with only ValueError: caught fully
run(ExceptionGroup("only_val", [ValueError("a"), ValueError("b")]))

# Group with TypeError only: propagates (no ValueError* match)
try:
    run(ExceptionGroup("only_type", [TypeError("t")]))
except* TypeError as eg:
    print("outer caught TypeError group, size", len(eg.exceptions))

# Mixed group: ValueError branch caught inside run,
# TypeError branch re-raised to outer handler.
try:
    run(ExceptionGroup("mixed", [ValueError("v"), TypeError("t")]))
except* TypeError as eg:
    print("outer caught TypeError group, size", len(eg.exceptions))