package websocket

import (
	"bufio"
	"bytes"
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
	ID   int
	Conn net.Conn
	Rw   *bufio.ReadWriter

	HandshakeHeaders http.Header
}

type Frame struct {
	fin     bool
	opcode  byte
	masked  byte
	pll     uint
	maskKey []byte
	payload []byte
}

func (ws *WebSocket) Handshake() error {
	hash := func(key string) string {
		h := sha1.New()
		h.Write([]byte(key))
		h.Write([]byte("258EAFA5-E914-47DA-95CA-C5AB0DC85B11"))
		return base64.StdEncoding.EncodeToString(h.Sum(nil))
	}(ws.HandshakeHeaders.Get("Sec-WebSocket-Key"))

	ws.Rw.WriteString(
		"HTTP/1.1 101 Switching Protocols\r\n" +
			"Upgrade: websocket\r\n" +
			"Connection: Upgrade\r\n" +
			fmt.Sprintf("Sec-WebSocket-Accept: %s\r\n", hash) +
			"\r\n")
	if err := ws.Rw.Flush(); err != nil {
		return fmt.Errorf("write flush failed: %q", err)
	}
	return nil
}

func (ws *WebSocket) Send(payload []byte) error {
	//ensure conn is active first
	_, err := ws.Rw.Write(payload)
	if err != nil {
		return fmt.Errorf("could not write to Conn: %q", err)
	}
	if err := ws.Rw.Flush(); err != nil {
		return fmt.Errorf("could not flush to Conn: %q", err)
	}

	return nil
}

func (ws *WebSocket) Recieve() error {
	//Set frame parsing
	var frame *Frame
	var payload []byte
	for {
		buf, err := ws.readBlock()
		if err != nil {
			return fmt.Errorf("readblock failed: %q", err)
		}
		logger.Printf("Incoming message (in bytes): % x", buf)

		frame, err = readToFrame(buf)
		if err != nil {
			return fmt.Errorf("parseFrame failed: %q", err)
		}
		payload = append(payload, frame.payload...)
		if frame.opcode == 0x08 {
			msg := responseFrame(frame)
			if err := ws.Send(msg); err != nil {
				return fmt.Errorf("could not respond to ws with close: %q", err)
			}
			ws.Conn.Close()
			lock.Lock()
			delete(Sockets, ws.ID)
			lock.Unlock()
			return nil
		}
		if frame.fin {
			break
		}
	}
	frame.payload = payload

	logger.Printf("Payload message (in string): %s", frame.payload)

	if err := broadcast(frame); err != nil {
		return fmt.Errorf("failed to broadcast to all connections: %q", err)
	}
	logger.Printf("Broadcast over. Going back to recieving")
	return nil
}

func broadcast(frame *Frame) error {
	msg := responseFrame(frame)

	lock.Lock()
	defer lock.Unlock()
	for _, ws := range Sockets {
		if err := ws.Send(msg); err != nil {
			return fmt.Errorf("could not send to ws: %q", err)
		}
	}
	return nil
}

func (ws *WebSocket) readBlock() ([]byte, error) {
	buf := make([]byte, 1024)

	for {
		n, err := ws.Rw.Read(buf)
		if err != nil {
			return nil, fmt.Errorf("connection read failed: %v", err)
		}
		if n > 0 {
			return buf[:n], nil
		}
	}
}

func readToFrame(data []byte) (*Frame, error) {
	//First byte
	var frame Frame
	counter := uint(0)

	frame.fin = data[counter]&0x80 == 0x80
	if data[counter]&0x70 != 0x00 {
		return nil, fmt.Errorf("rsv was provided")
	}

	frame.opcode = data[counter] & 0x0F
	counter++

	//PayloadLength
	frame.masked = data[counter] & 0x80
	masked := frame.masked == 0x80
	basePLL := data[counter] & 0x7F
	pll := uint(basePLL)
	if basePLL == 0x7E {
		counter++
		pll = uint(binary.BigEndian.Uint16(data[counter : counter+2]))
		counter = counter + 1
	}
	if basePLL == 0x7F {
		counter++
		pll = uint(binary.BigEndian.Uint64(data[counter : counter+8]))
		counter = counter + 7
	}
	frame.pll = pll
	counter++

	//Masking key and Payload
	payload := make([]byte, 0)
	if masked {
		maskKey := data[counter : counter+4]
		counter = counter + 4
		var j uint
		for i := uint(0); i < pll; i++ {
			j = i % 4
			unmasked := data[counter+i] ^ maskKey[j]
			payload = append(payload, unmasked)
		}
		frame.maskKey = maskKey
	} else {
		payload = append(payload, data[counter:]...)
	}

	frame.payload = payload

	return &frame, nil
}

// Creates a frame with partially copied data from specified frame
func responseFrame(frame *Frame) []byte {
	var buf bytes.Buffer
	//First byte
	buf.WriteByte(frame.opcode | 0b10000000)

	//pll
	if frame.pll <= 0x7D {
		buf.WriteByte(byte(frame.pll))
	} else if frame.pll >= 0x7E && frame.pll <= 0xFFFF {
		buf.WriteByte(0x7E)
		result := make([]byte, 4)
		binary.BigEndian.PutUint16(result, uint16(frame.pll))
		buf.Write(result)
	} else if frame.pll > 0xFFFF && frame.pll <= 0xFFFFFFFFFFFFFFFF {
		result := make([]byte, 8)
		buf.WriteByte(0x7F)
		binary.BigEndian.PutUint64(result, uint64(frame.pll))
		buf.Write(result)
	}

	//payload
	buf.Write(frame.payload)

	return buf.Bytes()
}
