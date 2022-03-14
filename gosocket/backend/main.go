package main

import (
	"fmt"
	"log"
	"net/http"
	"strings"
	"sync"
	"websocket/websocket"
)

var logger = log.Default()

var port = ":8080"
var counter = -1
var lock sync.Mutex

func main() {
	http.HandleFunc("/", httpHandler)
	logger.Printf("Starting http server @ port %s\n", port)
	http.ListenAndServe(port, nil)
}

func httpHandler(w http.ResponseWriter, r *http.Request) {
	ws, err := newConnection(w, r)
	if err != nil {
		logger.Fatal(err)
	}

	if err = ws.Handshake(); err != nil {
		logger.Fatal(err)
	}
	logger.Println("Handshake over!")

	websocket.Sockets[ws.ID] = *ws

	for {
		if err := ws.Recieve(); err != nil {
			logger.Printf("Websocket recieve got: %v", err)
			break
		}
	}
}

func newConnection(w http.ResponseWriter, r *http.Request) (*websocket.WebSocket, error) {
	hj, ok := w.(http.Hijacker)
	if !ok {
		return nil, fmt.Errorf("could recast writer to  http.hijack")
	}

	conn, rw, err := hj.Hijack()
	if err != nil {
		return nil, fmt.Errorf("could not hijak connection: %q", err)
	}

	if err = verifyHeaders(r); err != nil {
		return nil, fmt.Errorf("could not verify necessary websocket headers: %q", err)
	}
	lock.Lock()
	defer lock.Unlock()
	counter++
	return &websocket.WebSocket{ID: counter, Conn: conn, Rw: rw, HandshakeHeaders: r.Header}, nil
}

func verifyHeaders(r *http.Request) error {
	method := "GET"
	//host := "localhost" + port
	upgrade := "websocket"
	connection := "Upgrade"
	version := "13"

	if r.Method != method {
		return fmt.Errorf("invalid method provided. Must be %s", method)
	}
	// if r.Host != host {
	// 	return fmt.Errorf("invalid host provided. Must be %s", port)
	// }
	if r.Header.Get("Upgrade") != upgrade {
		return fmt.Errorf("invalid 'Upgrade' provided. Must be '%s'", upgrade)
	}
	if !strings.Contains(r.Header.Get("Connection"), connection) {
		logger.Printf("Got failing value: %s", r.Header.Get("Connection"))
		return fmt.Errorf("invalid 'Connection' provided. Must be '%s'", connection)
	}
	key := r.Header.Get("Sec-WebSocket-Key")
	if key == "" {
		return fmt.Errorf("'Sec-WebSocket-Key' not provided/found")
	}
	origin := r.Header.Get("Origin")
	if origin == "" {
		return fmt.Errorf("'Origin' was not provided/found")
	}
	if r.Header.Get("Sec-WebSocket-Version") != version {
		return fmt.Errorf("invalid 'Sec-WebSocket-Version' provided. Must be '%s'", version)
	}

	return nil
}
