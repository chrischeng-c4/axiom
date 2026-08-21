// go_suite template_render shape -- Go twin of ../../../fixtures/template_render.py.
//
// Server-shaped: build a small "order confirmation" text body per record via
// string concatenation/joins.
package main

import (
	"fmt"
	"strconv"
	"strings"
)

type Item struct {
	SKU        string
	Qty        int
	PriceCents int
}

type Order struct {
	OrderID  int
	Customer string
	Items    []Item
}

func buildOrders(n int) []Order {
	skus := []string{"SKU-A1", "SKU-B2", "SKU-C3", "SKU-D4", "SKU-E5"}
	out := make([]Order, 0, n)
	for i := 0; i < n; i++ {
		itemCount := 1 + (i % 4)
		items := make([]Item, 0, itemCount)
		for j := 0; j < itemCount; j++ {
			sku := skus[(i+j)%5]
			qty := 1 + ((i*3 + j) % 5)
			price := 500 + ((i*17 + j*31) % 4500)
			items = append(items, Item{SKU: sku, Qty: qty, PriceCents: price})
		}
		out = append(out, Order{OrderID: i, Customer: "customer-" + strconv.Itoa(i%300), Items: items})
	}
	return out
}

func renderOrder(o Order) string {
	lines := make([]string, 0, len(o.Items)+2)
	lines = append(lines, "Order #"+strconv.Itoa(o.OrderID)+" for "+o.Customer)
	totalCents := 0
	for _, it := range o.Items {
		lineTotal := it.Qty * it.PriceCents
		totalCents += lineTotal
		lines = append(lines, "  "+it.SKU+" x"+strconv.Itoa(it.Qty)+" @ "+strconv.Itoa(it.PriceCents)+
			"c = "+strconv.Itoa(lineTotal)+"c")
	}
	lines = append(lines, "Total: "+strconv.Itoa(totalCents)+"c")
	return strings.Join(lines, "\n")
}

func checksum(data []byte) uint64 {
	var h uint64 = 0
	const mod uint64 = 1000000007
	const mult uint64 = 131
	for _, b := range data {
		h = (h*mult + uint64(b)) % mod
	}
	return h
}

func main() {
	orders := buildOrders(1500)
	renderedParts := make([]string, 0, len(orders))
	for _, o := range orders {
		renderedParts = append(renderedParts, renderOrder(o))
	}
	full := strings.Join(renderedParts, "\n---\n")
	fmt.Println("CHECKSUM", checksum([]byte(full)))
}
