package main

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"sync"

	"github.com/skip2/go-qrcode"
	_ "github.com/mattn/go-sqlite3"
	"go.mau.fi/whatsmeow"
	"go.mau.fi/whatsmeow/proto/waE2E"
	"go.mau.fi/whatsmeow/store/sqlstore"
	"go.mau.fi/whatsmeow/types"
	"go.mau.fi/whatsmeow/types/events"
	waLog "go.mau.fi/whatsmeow/util/log"
)

type Command struct {
	Action string `json:"action"`
	To     string `json:"to"`
	Text   string `json:"text"`
}

type IncomingMessage struct {
	Action    string `json:"action"`
	From      string `json:"from"`
	Text      string `json:"text"`
	Timestamp string `json:"timestamp"`
}

var client *whatsmeow.Client
var pushConns []net.Conn
var pushMu sync.Mutex

func main() {
	dbLog := waLog.Noop
	container, err := sqlstore.New(context.Background(), "sqlite3", "file:session.db?_foreign_keys=on&_busy_timeout=5000", dbLog)
	if err != nil {
		panic(err)
	}

	deviceStore, err := container.GetFirstDevice(context.Background())
	if err != nil {
		panic(err)
	}

	clientLog := waLog.Noop
	client = whatsmeow.NewClient(deviceStore, clientLog)

	// register incoming message handler BEFORE connecting
	client.AddEventHandler(handleWhatsAppEvent)

	if client.Store.ID == nil {
		qrChan, _ := client.GetQRChannel(context.Background())
		err = client.Connect()
		if err != nil {
			panic(err)
		}
		for evt := range qrChan {
			if evt.Event == "code" {
				fmt.Println("Scan this QR code with WhatsApp:")
				qr, err := qrcode.New(evt.Code, qrcode.Medium)
				if err != nil {
					fmt.Println("QR error:", err)
					continue
				}
				fmt.Println(qr.ToSmallString(false))
			} else {
				fmt.Println("Login event:", evt.Event)
			}
		}
	} else {
		err = client.Connect()
		if err != nil {
			panic(err)
		}
	}

	redirectOutput()
	os.WriteFile("/tmp/seep-ready", []byte("ready"), 0644)

	go listenForPush()      // ← push socket for incoming messages
	listenForCommands()     // ← command socket (blocks)
}

func handleWhatsAppEvent(evt interface{}) {
	switch v := evt.(type) {
	case *events.Message:
		if v.Info.IsFromMe {
			return
		}

		// extract text from different message types
		text := ""
		if v.Message.GetConversation() != "" {
			text = v.Message.GetConversation()
		} else if v.Message.ExtendedTextMessage != nil {
			text = v.Message.ExtendedTextMessage.GetText()
		}
		if text == "" {
			return
		}

		msg := IncomingMessage{
			Action:    "message",
			From:      v.Info.Sender.String(),
			Text:      text,
			Timestamp: v.Info.Timestamp.Format("15:04"),
		}
		data, _ := json.Marshal(msg)
		broadcast(data)
	}
}

func addPushConn(conn net.Conn) {
	pushMu.Lock()
	defer pushMu.Unlock()
	pushConns = append(pushConns, conn)
}

func broadcast(data []byte) {
	pushMu.Lock()
	defer pushMu.Unlock()
	var alive []net.Conn
	for _, c := range pushConns {
		_, err := c.Write(append(data, '\n'))
		if err == nil {
			alive = append(alive, c)
		}
	}
	pushConns = alive
}

func listenForPush() {
	socketPath := "/tmp/seep-push.sock"
	os.Remove(socketPath)
	listener, err := net.Listen("unix", socketPath)
	if err != nil {
		return
	}
	defer listener.Close()
	for {
		conn, err := listener.Accept()
		if err != nil {
			continue
		}
		addPushConn(conn) // just register, Go pushes to it
	}
}

func listenForCommands() {
	socketPath := "/tmp/seep-bridge.sock"
	os.Remove(socketPath)
	listener, err := net.Listen("unix", socketPath)
	if err != nil {
		panic(err)
	}
	defer listener.Close()
	for {
		conn, err := listener.Accept()
		if err != nil {
			continue
		}
		go handleConnection(conn)
	}
}

func handleConnection(conn net.Conn) {
	defer conn.Close()
	scanner := bufio.NewScanner(conn)
	for scanner.Scan() {
		line := scanner.Text()
		var cmd Command
		if err := json.Unmarshal([]byte(line), &cmd); err != nil {
			continue
		}
		switch cmd.Action {
		case "send":
			sendMessage(cmd.To, cmd.Text)
		case "get_contacts":
			data, err := getContacts()
			if err != nil {
				continue
			}
			conn.Write(append(data, '\n'))
		}
	}
}

func sendMessage(to, text string) {
	jid, err := types.ParseJID(to)
	if err != nil {
		return
	}
	client.SendMessage(context.Background(), jid, &waE2E.Message{
		Conversation: &text,
	})
}

func getContacts() ([]byte, error) {
	contacts, err := client.Store.Contacts.GetAllContacts(context.Background())
	if err != nil {
		return nil, err
	}

	type Contact struct {
		Name string `json:"name"`
		JID  string `json:"jid"`
	}

	var result []Contact
	for jid, contact := range contacts {
		name := contact.FullName
		if name == "" {
			name = contact.PushName
		}
		if name == "" {
			continue
		}
		result = append(result, Contact{Name: name, JID: jid.String()})
	}
	return json.Marshal(result)
}

func redirectOutput() {
	logFile, err := os.OpenFile("/tmp/seep-bridge.log", os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0644)
	if err != nil {
		return
	}
	os.Stdout = logFile
	os.Stderr = logFile
}
