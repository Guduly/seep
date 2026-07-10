package main

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"net"
	"os"

	_ "github.com/mattn/go-sqlite3"
	"go.mau.fi/whatsmeow"
	"go.mau.fi/whatsmeow/store/sqlstore"
	"go.mau.fi/whatsmeow/types"
	"github.com/skip2/go-qrcode"
	"go.mau.fi/whatsmeow/proto/waE2E"
	waLog "go.mau.fi/whatsmeow/util/log"
)

// Message from Rust to Go
type Command struct {
	Action string `json:"action"`
	To     string `json:"to"`
	Text   string `json:"text"`
}

// Message from Go to Rust
type Event struct {
	From      string `json:"from"`
	Text      string `json:"text"`
	Timestamp string `json:"timestamp"`
}

var client *whatsmeow.Client

func main() {
	// setup database to store session
	dbLog := waLog.Stdout("Database", "ERROR", true)
	container, err := sqlstore.New(context.Background(), "sqlite3", "file:session.db?_foreign_keys=on&_busy_timeout=5000", dbLog)
	if err != nil {
		panic(err)
	}

	// get or create device
	deviceStore, err := container.GetFirstDevice(context.Background())
	if err != nil {
		panic(err)
	}

	// create whatsapp client
	clientLog := waLog.Stdout("Client", "ERROR", true)
	client = whatsmeow.NewClient(deviceStore, clientLog)

	// login
		// login
	if client.Store.ID == nil {
			// first run - show QR code
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
			// already logged in - just connect
			err = client.Connect()
			if err != nil {
					panic(err)
			}
	}
	fmt.Println("Connected to WhatsApp!")

	// start listening for commands from Rust
	listenForCommands()
}

func listenForCommands() {
	// create unix socket
	socketPath := "/tmp/seep-bridge.sock"
	os.Remove(socketPath) // remove old socket if exists

	listener, err := net.Listen("unix", socketPath)
	if err != nil {
		panic(err)
	}
	defer listener.Close()

	fmt.Println("Bridge ready, waiting for commands...")

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
        fmt.Println("Received command:", line)  // ← add this
        var cmd Command
        if err := json.Unmarshal([]byte(line), &cmd); err != nil {
            fmt.Println("JSON error:", err)  // ← and this
            continue
        }
        fmt.Println("Parsed action:", cmd.Action, "to:", cmd.To)  // ← and this
        switch cmd.Action {
        case "send":
            sendMessage(cmd.To, cmd.Text)
				case  "get_contacts": 
						data,err := getContacts()
						if err != nil{
							fmt.Println("Error Getting Contacts", err)
							continue
						}
						conn.Write(append(data, '\n'))
        }
    }
    fmt.Println("Scanner error:", scanner.Err())  // ← and this
}

func sendMessage(to, text string) {
    fmt.Println("Attempting to send to:", to)  // ← add this
    jid, err := types.ParseJID(to)
    if err != nil {
        fmt.Println("Invalid JID:", err)
        return
    }
    fmt.Println("Sending message...")  // ← add this
    _, err = client.SendMessage(context.Background(), jid, &waE2E.Message{
        Conversation: &text,
    })
    if err != nil {
        fmt.Println("Send error:", err)  // ← add this
    } else {
        fmt.Println("Message sent successfully!")
    }
}

func getContacts() ([] byte, error){
	contacts, err := client.Store.Contacts.GetAllContacts(context.Background())

	if err != nil{
		  return nil, err
	}

	type Contact struct{
		  Name string `json:"name"`
			JID string `json:"jid"`
	}

	var result []Contact

	for jid, contact :=  range contacts{
		  name := contact.FullName 
			if name == ""{
			   name  = contact.PushName
			}

			if name == "" {continue}

			result = append(result, Contact{
				Name: name, 
				JID: jid.String(),
			})
	}

	return json.Marshal(result)
}
