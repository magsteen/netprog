package websocket

import (
	"bufio"
	"crypto/sha1"
	"encoding/base64"
	"encoding/binary"
	"fmt"
	"log"
	"net"
	"net/http"
	"sync"
)

var Sockets = make(map[int]WebSocket)
var lock = sync.Mutex{}

var logger = log.Default()

type WebSocket struct {
	Conn net.Conn
	Rw   *bufio.ReadWriter

	HandshakeHeaders http.Header
}

func (ws *WebSocket) Handshake() error {

	hash := func(key string) string {
		h := sha1.New()
		h.Write([]byte(key))
		h.Write([]byte("258EAFA5-E914-47DA-95CA-C5AB0DC85B11"))
		return base64.StdEncoding.EncodeToString(h.Sum(nil))
	}(ws.HandshakeHeaders.Get("Sec-WebSocket-Key"))

	ws.Rw.WriteString("HTTP/1.1 101 Switching Protocols\r\n")
	ws.Rw.WriteString("Upgrade: websocket\r\n")
	ws.Rw.WriteString("Connection: Upgrade\r\n")
	ws.Rw.WriteString(fmt.Sprintf("Sec-WebSocket-Accept: %s\r\n", hash))
	ws.Rw.WriteString("\r\n")

	if err := ws.Rw.Flush(); err != nil {
		return fmt.Errorf("write flush failed: %q", err)
	}
	return nil
}

func (ws *WebSocket) Send(payload []byte) error {
	//Set up a frame with the specified paylaod
	msg := []byte{}

	ws.Conn.Write(msg)
	return nil
}

func (ws *WebSocket) Recieve() error {
	//Set frame parsing
	buf, err := ws.readBlock()
	if err != nil {
		return fmt.Errorf("readblock failed: %q", err)
	}
	logger.Printf("Message: % x", buf)

	msg, err := parseFrame(buf)
	if err != nil {
		return fmt.Errorf("parseFrame failed: %q", err)
	}

	if err = broadcast(msg); err != nil {
		return fmt.Errorf("failed to broadcast to all connections: %q", err)
	}
	return nil
}

func broadcast(msg []byte) error {
	lock.Lock()
	defer lock.Unlock()
	for _, ws := range Sockets {
		if err := ws.Send(msg); err != nil {
			return fmt.Errorf("could not send to connection: %q", err)
		}
	}
	return nil
}

func (ws *WebSocket) readBlock() ([]byte, error) {
	buf := make([]byte, 512)

	for {
		n, err := ws.Rw.Read(buf)
		if err != nil {
			return nil, fmt.Errorf("connection read failed: %q", err)
		}
		if n > 0 {
			return buf[:n], nil
		}
	}
}

func parseFrame(frame []byte) ([]byte, error) {
	msg := []byte{}
	counter := 0

	//First byte
	fmt.Printf("%08b\n", frame[0])
	if frame[0]&0x70 != 0x00 {
		return nil, fmt.Errorf("rsv was provided")
	}
	handleOpcode(frame[0] & 0x0F)

	//Second byte
	fmt.Printf("%08b\n", frame[1])

	masked := frame[1]&0x80 == 0x80
	fmt.Printf("%t\n", masked)

	pll := int(frame[1] & 0x79)
	counter++
	fmt.Printf("PLL: %d\n", pll)
	// if basePLL >= 0x00 && basePLL <= 0x7D {

	// }
	if frame[1] == 0x7E {
		fmt.Printf("%08b\n", frame[counter:2])
		pll = int(binary.LittleEndian.Uint16(frame[counter:2]))
		fmt.Printf("PLL: %d\n", pll)
	}
	if frame[1] == 0x7F {
		pll = int(binary.LittleEndian.Uint16(frame[counter:8]))
		fmt.Printf("PLL: %d\n", pll)
	}
	return msg, nil
}

func handleOpcode(opcode byte) {

}
