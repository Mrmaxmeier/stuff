package main

import (
	"fmt"
	"net"
	"os"
	"os/exec"
	"os/signal"
	"time"

	"github.com/bobziuchkovski/writ"
)

const sockfile = "/tmp/lock.sock"

const (
	invalidMsg = iota
	locknow
	suspendnow
)

type Config struct {
	HelpFlag     bool   `flag:"help" description:"Display this help message and exit"`
	Verbosity    int    `flag:"v, verbose" description:"Display verbose output"`
	Progress     bool   `flag:"progress" description:"Display status info line"`
	Lock         bool   `flag:"lock" description:"Lock now"`
	Suspend      bool   `flag:"suspend" description:"Suspend now"`
	LockAfter    string `option:"lock-after" default:"3m"`
	SuspendAfter string `option:"suspend-after" default:"5m"`
}

func main() {
	config := &Config{}
	cmd := writ.New("lock", config)
	cmd.Help.Usage = "Usage: lock [OPTION]..."
	cmd.Help.Header = "Some crappy locker."
	_, _, err := cmd.Decode(os.Args[1:])
	if err != nil || config.HelpFlag {
		fmt.Println(`
                          / '.   .' "
                  .---.  <    > <    >  .---.
                  |    \  \ - ~ ~ - /  /    |
      _____          ..-~             ~-..-~
     |     |   \~~~\.'                    './~~~/
    ---------   \__/                        \__/
   .'  O    \     /              /        \  "
  (_____,    '._.'               |         }  \/~~~/
   '----.          /       }     |        /    \__/
         '-.      |       /      |       /      '. ,~~|
             ~-.__|      /_ - ~ ^|      /- _      '..-'
                  |     /        |     /     ~-.     '-. _  _  _
                  |_____|        |_____|         ~ - . _ _ _ _ _>
		`)
		cmd.ExitHelp(err)
	}

	if config.Lock || config.Suspend {
		con, err := net.Dial("unix", sockfile)
		if err != nil {
			notify(err.Error())
			return
		}
		if config.Lock {
			notify("locking remotely...")
			con.Write([]byte{byte(locknow)})
		} else {
			notify("suspending remotely...")
			con.Write([]byte{byte(suspendnow)})
		}
		con.Close()
		return
	}

	getDur := func(s string) time.Duration {
		time, err := time.ParseDuration(s)
		if err != nil {
			panic(err)
		}
		return time
	}
	locker := I3Lock{}
	locker.Init()
	idletrigger := XIdleTrigger{
		lock:    getDur(config.LockAfter),
		suspend: getDur(config.SuspendAfter),
		locker:  &locker,
	}
	go listen(&locker)
	idletrigger.Run(config)
}

func listen(locker Locker) {
	l, err := net.Listen("unix", sockfile)
	if err != nil {
		fmt.Println("listen error", err)
		return
	}

	go func() {
		for {
			fd, err := l.Accept()
			if err != nil {
				notify(err.Error())
				continue
			}
			handleClient(fd, locker)
		}
	}()

	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt)
	<-c
	l.Close()
	os.Exit(1)
}

func handleClient(fd net.Conn, locker Locker) {
	command := make([]byte, 1)
	_, err := fd.Read(command)
	if err != nil {
		notify(err.Error())
		return
	}
	switch command[0] {
	case locknow:
		go func() {
			fmt.Println("locking due to unixsock", fd)
			locker.Lock()
		}()
	case suspendnow:
		go func() {
			fmt.Println("suspending due to unixsock", fd)
			suspend()
		}()
	case invalidMsg:
	default:
		notify("invalid message!")
	}
}

func notify(msg string) {
	fmt.Println(msg)
	exec.Command("notify-send", msg).Run()
}

func suspend() {
	err := exec.Command("systemctl", "suspend").Run()
	if err != nil {
		notify(err.Error())
	}
	time.Sleep(time.Second * 5) // TODO: magic resume detection
}
