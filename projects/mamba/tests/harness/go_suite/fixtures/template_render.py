"""go_suite shape: template-ish string rendering.

Server-shaped: build a small "order confirmation" text body per record via
string concatenation/joins (no templating library -- the manual-render path
every server framework falls back to under a hot loop). Typed: `Item`/`Order`
fields, `list[str]` line accumulator per render.
"""


class Item:
    def __init__(self, sku: str, qty: int, price_cents: int) -> None:
        self.sku: str = sku
        self.qty: int = qty
        self.price_cents: int = price_cents


class Order:
    def __init__(self, order_id: int, customer: str, items: list[Item]) -> None:
        self.order_id: int = order_id
        self.customer: str = customer
        self.items: list[Item] = items


def build_orders(n: int) -> list[Order]:
    skus = ["SKU-A1", "SKU-B2", "SKU-C3", "SKU-D4", "SKU-E5"]
    out: list[Order] = []
    for i in range(n):
        items: list[Item] = []
        item_count = 1 + (i % 4)
        for j in range(item_count):
            sku = skus[(i + j) % 5]
            qty = 1 + ((i * 3 + j) % 5)
            price = 500 + ((i * 17 + j * 31) % 4500)
            items.append(Item(sku, qty, price))
        out.append(Order(i, "customer-" + str(i % 300), items))
    return out


def render_order(o: Order) -> str:
    lines: list[str] = []
    lines.append("Order #" + str(o.order_id) + " for " + o.customer)
    total_cents = 0
    for it in o.items:
        line_total = it.qty * it.price_cents
        total_cents += line_total
        lines.append(
            "  " + it.sku + " x" + str(it.qty) + " @ " + str(it.price_cents)
            + "c = " + str(line_total) + "c"
        )
    lines.append("Total: " + str(total_cents) + "c")
    return "\n".join(lines)


def checksum(data: bytes) -> int:
    h: int = 0
    mod: int = 1000000007
    mult: int = 131
    for b in data:
        h = (h * mult + b) % mod
    return h


def main() -> None:
    orders = build_orders(1500)
    rendered_parts: list[str] = []
    for o in orders:
        rendered_parts.append(render_order(o))
    full = "\n---\n".join(rendered_parts)
    print("CHECKSUM", checksum(full.encode("utf-8")))


main()
