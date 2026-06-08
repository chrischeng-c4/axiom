// spectral-norm (Computer Language Benchmarks Game) — idiomatic Go.
// Same algorithm and same N as the sibling spectral_norm.py, for a like-for-like
// python / go / mamba comparison. No unsafe / SIMD intrinsics — plain Go.
package main

import (
	"fmt"
	"math"
)

const N = 1000

func evalA(i, j int) float64 {
	return 1.0 / float64((i+j)*(i+j+1)/2+i+1)
}

func evalATimesU(u []float64) []float64 {
	n := len(u)
	v := make([]float64, n)
	for i := 0; i < n; i++ {
		var s float64
		for j := 0; j < n; j++ {
			s += evalA(i, j) * u[j]
		}
		v[i] = s
	}
	return v
}

func evalATTimesU(u []float64) []float64 {
	n := len(u)
	v := make([]float64, n)
	for i := 0; i < n; i++ {
		var s float64
		for j := 0; j < n; j++ {
			s += evalA(j, i) * u[j]
		}
		v[i] = s
	}
	return v
}

func evalAtATimesU(u []float64) []float64 {
	return evalATTimesU(evalATimesU(u))
}

func main() {
	u := make([]float64, N)
	for i := range u {
		u[i] = 1.0
	}
	v := u
	for k := 0; k < 10; k++ {
		v = evalAtATimesU(u)
		u = evalAtATimesU(v)
	}
	var vBv, vv float64
	for i := 0; i < N; i++ {
		vBv += u[i] * v[i]
		vv += v[i] * v[i]
	}
	fmt.Printf("%.9f\n", math.Sqrt(vBv/vv))
}
