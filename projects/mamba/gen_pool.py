import sys

def gen(n, indent="    "):
    ids = [f"shell_{i:02d}" for i in range(n)]
    # wrap into lines of ~8 per line like long_tail_mod style
    lines = []
    for i in range(0, len(ids), 9):
        chunk = ids[i:i+9]
        lines.append(indent + ", ".join(chunk) + ("," if i+9 < len(ids) else ","))
    return "\n".join(lines)

if __name__ == "__main__":
    n = int(sys.argv[1])
    print(gen(n))
