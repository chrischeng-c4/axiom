// go_suite json_codec shape -- Go twin of ../../../fixtures/json_codec.py.
//
// Server-shaped: build typed records, round-trip them through JSON
// (marshal -> unmarshal), and fold the decoded fields into a checksum. Kept
// field-for-field identical to the Python twin: same record generation, same
// iteration count, same fold order, same final mod.
package main

import (
	"encoding/json"
	"fmt"
	"strconv"
)

type Record struct {
	ID    int      `json:"id"`
	Name  string   `json:"name"`
	Tags  []string `json:"tags"`
	Score int      `json:"score"`
}

func buildRecords(n int) []Record {
	tagPool := []string{"alpha", "beta", "gamma", "delta", "epsilon"}
	out := make([]Record, 0, n)
	for i := 0; i < n; i++ {
		tags := []string{tagPool[i%5], tagPool[(i*3)%5]}
		out = append(out, Record{ID: i, Name: "item-" + strconv.Itoa(i), Tags: tags, Score: (i * 37) % 1000})
	}
	return out
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
	records := buildRecords(500)
	iterations := 30
	var total int64 = 0
	for iter := 0; iter < iterations; iter++ {
		payload := make([]Record, len(records))
		copy(payload, records)
		text, err := json.Marshal(payload)
		if err != nil {
			panic(err)
		}
		var parsed []Record
		if err := json.Unmarshal(text, &parsed); err != nil {
			panic(err)
		}
		for _, obj := range parsed {
			total += int64(obj.ID)
			total += int64(obj.Score)
			total += int64(len(obj.Name))
			for _, tag := range obj.Tags {
				total += int64(len(tag))
			}
		}
		total = total % 1000000007
	}
	summary := "json_codec:" + strconv.FormatInt(total, 10)
	fmt.Println("CHECKSUM", checksum([]byte(summary)))
}
