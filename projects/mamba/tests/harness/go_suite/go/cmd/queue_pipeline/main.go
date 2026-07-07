// go_suite queue_pipeline shape -- Go twin of ../../../fixtures/queue_pipeline.py.
//
// Server-shaped: a bounded FIFO work queue where producers enqueue typed
// tasks and workers dequeue + process them. Deliberately single-threaded to
// mirror the Python twin's determinism (see that file's docstring for why);
// the queue itself is a small hand-rolled FIFO over a slice, exercising the
// same put/get/full/empty operations `queue.Queue` provides on the Python
// side without pulling in goroutine scheduling (out of this epic's scope).
package main

import (
	"fmt"
	"strconv"
)

type Task struct {
	TaskID   int
	Priority int
	Payload  int
}

type FIFOQueue struct {
	items   []Task
	maxsize int
}

func NewFIFOQueue(maxsize int) *FIFOQueue {
	return &FIFOQueue{items: make([]Task, 0, maxsize), maxsize: maxsize}
}

func (q *FIFOQueue) Full() bool  { return len(q.items) >= q.maxsize }
func (q *FIFOQueue) Empty() bool { return len(q.items) == 0 }

func (q *FIFOQueue) Put(t Task) {
	q.items = append(q.items, t)
}

func (q *FIFOQueue) Get() Task {
	t := q.items[0]
	q.items = q.items[1:]
	return t
}

func processTask(t Task) int {
	return (t.Payload*31 + t.Priority*7 + t.TaskID) % 1000003
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
	nTasks := 4000
	maxInflight := 64
	q := NewFIFOQueue(maxInflight)

	produced := 0
	consumed := 0
	results := make([]int, 0, nTasks)

	for consumed < nTasks {
		for produced < nTasks && !q.Full() {
			t := Task{TaskID: produced, Priority: produced % 5, Payload: (produced * 17) % 10007}
			q.Put(t)
			produced++
		}
		drainedThisRound := 0
		for !q.Empty() && drainedThisRound < 32 {
			t := q.Get()
			results = append(results, processTask(t))
			consumed++
			drainedThisRound++
		}
	}

	total := 0
	for _, r := range results {
		total = (total + r) % 1000000007
	}
	summary := "queue_pipeline:" + strconv.Itoa(produced) + ":" + strconv.Itoa(consumed) + ":" + strconv.Itoa(total)
	fmt.Println("CHECKSUM", checksum([]byte(summary)))
}
