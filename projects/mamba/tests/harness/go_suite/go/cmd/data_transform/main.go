// go_suite data_transform shape -- Go twin of ../../../fixtures/data_transform.py.
//
// Server-shaped: filter -> map -> group-by -> aggregate over a stream of
// typed event records.
package main

import (
	"fmt"
	"sort"
	"strconv"
	"strings"
)

type Event struct {
	UserID    int
	EventType string
	Value     int
	Ts        int
}

func buildEvents(n int) []Event {
	types := []string{"click", "view", "purchase", "refund", "signup"}
	out := make([]Event, 0, n)
	for i := 0; i < n; i++ {
		out = append(out, Event{
			UserID:    i % 200,
			EventType: types[i%5],
			Value:     (i * 13) % 500,
			Ts:        1700000000 + i,
		})
	}
	return out
}

func transform(events []Event) map[int][]int {
	groups := map[int][]int{}
	for _, e := range events {
		if e.EventType == "refund" {
			continue
		}
		weight := e.Value
		if e.EventType == "purchase" {
			weight = weight * 10
		}
		groups[e.UserID] = append(groups[e.UserID], weight)
	}
	return groups
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
	events := buildEvents(20000)
	groups := transform(events)
	keys := make([]int, 0, len(groups))
	for k := range groups {
		keys = append(keys, k)
	}
	sort.Ints(keys)
	parts := make([]string, 0, len(keys))
	for _, uid := range keys {
		vs := groups[uid]
		total := 0
		mx := vs[0]
		mn := vs[0]
		for _, v := range vs {
			total += v
			if v > mx {
				mx = v
			}
			if v < mn {
				mn = v
			}
		}
		cnt := len(vs)
		parts = append(parts, strconv.Itoa(uid)+":"+strconv.Itoa(total)+":"+strconv.Itoa(cnt)+":"+strconv.Itoa(mx)+":"+strconv.Itoa(mn))
	}
	summary := "data_transform|" + strings.Join(parts, ";")
	fmt.Println("CHECKSUM", checksum([]byte(summary)))
}
