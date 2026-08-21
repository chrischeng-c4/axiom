// go_suite string_processing shape -- Go twin of ../../../fixtures/string_processing.py.
//
// Server-shaped: tokenize a corpus and build a word-frequency table. The
// corpus generator is the identical small-constant LCG as the Python twin
// (seed*48271 % 2147483647, a textbook Lehmer/Park-Miller MINSTD generator)
// so both languages walk the exact same vocabulary-index sequence.
package main

import (
	"fmt"
	"sort"
	"strconv"
	"strings"
)

var vocab = []string{
	"the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog", "server", "request",
	"response", "handler", "route", "payload", "json", "queue", "worker", "cache", "token", "session",
	"database", "query", "index", "latency", "throughput", "cpu", "memory", "thread", "process", "socket",
}

func genCorpus(nWords int) []string {
	seed := 12345
	words := make([]string, 0, nWords)
	for i := 0; i < nWords; i++ {
		seed = (seed * 48271) % 2147483647
		idx := seed % len(vocab)
		words = append(words, vocab[idx])
	}
	return words
}

func wordCounts(words []string) map[string]int {
	counts := map[string]int{}
	for _, w := range words {
		counts[w] = counts[w] + 1
	}
	return counts
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
	words := genCorpus(30000)
	counts := wordCounts(words)
	keys := make([]string, 0, len(counts))
	for k := range counts {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	parts := make([]string, 0, len(keys))
	for _, k := range keys {
		parts = append(parts, k+":"+strconv.Itoa(counts[k]))
	}
	summary := "string_processing|" + strings.Join(parts, "|")
	fmt.Println("CHECKSUM", checksum([]byte(summary)))
}
