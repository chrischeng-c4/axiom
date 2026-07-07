// go_suite route_match shape -- Go twin of ../../../fixtures/route_match.py.
//
// Server-shaped: a small static+param route table matched against a stream
// of request paths, segment-by-segment, extracting path params for matches.
package main

import (
	"fmt"
	"sort"
	"strconv"
	"strings"
)

type Route struct {
	RouteID  int
	Pattern  string
	Segments []string
}

func buildRoutes() []Route {
	patterns := []string{
		"/health",
		"/metrics",
		"/api/v1/users",
		"/api/v1/users/:id",
		"/api/v1/users/:id/orders",
		"/api/v1/users/:id/orders/:oid",
		"/api/v1/products",
		"/api/v1/products/:id",
		"/api/v1/products/:id/reviews",
		"/api/v1/products/:id/reviews/:rid",
		"/api/v2/search",
		"/api/v2/search/:query",
		"/admin/dashboard",
		"/admin/settings/:section",
	}
	routes := make([]Route, 0, len(patterns))
	for i, p := range patterns {
		segs := strings.Split(strings.Trim(p, "/"), "/")
		routes = append(routes, Route{RouteID: i, Pattern: p, Segments: segs})
	}
	return routes
}

func matchRoute(routes []Route, path string) (int, map[string]string) {
	segs := strings.Split(strings.Trim(path, "/"), "/")
	for _, r := range routes {
		if len(r.Segments) != len(segs) {
			continue
		}
		params := map[string]string{}
		ok := true
		for i, seg := range r.Segments {
			if len(seg) > 0 && seg[0] == ':' {
				params[seg[1:]] = segs[i]
			} else if seg != segs[i] {
				ok = false
				break
			}
		}
		if ok {
			return r.RouteID, params
		}
	}
	return -1, map[string]string{}
}

func buildRequests(n int) []string {
	templates := []string{
		"/health",
		"/metrics",
		"/api/v1/users",
		"/api/v1/users/{}",
		"/api/v1/users/{}/orders",
		"/api/v1/users/{}/orders/{}",
		"/api/v1/products",
		"/api/v1/products/{}",
		"/api/v1/products/{}/reviews",
		"/api/v1/products/{}/reviews/{}",
		"/api/v2/search",
		"/api/v2/search/{}",
		"/admin/dashboard",
		"/admin/settings/{}",
		"/not/a/real/route/at/all",
	}
	out := make([]string, 0, n)
	for i := 0; i < n; i++ {
		t := templates[i%len(templates)]
		filled := strings.Replace(t, "{}", strconv.Itoa(i%97), 1)
		if strings.Contains(filled, "{}") {
			filled = strings.Replace(filled, "{}", strconv.Itoa((i*7)%53), 1)
		}
		out = append(out, filled)
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
	routes := buildRoutes()
	requests := buildRequests(3000)
	matchedCount := 0
	parts := make([]string, 0, len(requests))
	for _, path := range requests {
		routeID, params := matchRoute(routes, path)
		if routeID >= 0 {
			matchedCount++
			keys := make([]string, 0, len(params))
			for k := range params {
				keys = append(keys, k)
			}
			sort.Strings(keys)
			pairs := make([]string, 0, len(keys))
			for _, k := range keys {
				pairs = append(pairs, k+"="+params[k])
			}
			parts = append(parts, strconv.Itoa(routeID)+":"+strings.Join(pairs, ","))
		} else {
			parts = append(parts, "nomatch")
		}
	}
	summary := "route_match:" + strconv.Itoa(matchedCount) + "|" + strings.Join(parts, ";")
	fmt.Println("CHECKSUM", checksum([]byte(summary)))
}
