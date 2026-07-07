// go_suite hello shape -- Go twin of ../../../fixtures/hello.py.
//
// Minimal fixture used ONLY to measure process startup: wall-clock time from
// process spawn to the first stdout byte. See the Python twin's docstring
// for why it stays trivial.
package main

import "fmt"

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
	msg := "hello-go-suite"
	fmt.Println(msg)
	fmt.Println("CHECKSUM", checksum([]byte(msg)))
}
