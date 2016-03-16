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
	lockMsg
	suspendMsg
	quitMsg
)

var signalChan chan os.Signal

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
			con.Write([]byte{byte(lockMsg)})
		} else {
			notify("suspending remotely...")
			con.Write([]byte{byte(suspendMsg)})
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
	l, err = checkListenErrorCloseOther(l, err)
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

	signalChan = make(chan os.Signal, 1)
	signal.Notify(signalChan, os.Interrupt)
	<-signalChan
	fmt.Println("got interrupt signal")
	l.Close()
	os.Exit(1)
}

func checkListenErrorCloseOther(l net.Listener, err error) (net.Listener, error) {
	switch err := err.(type) {
	case *net.OpError:
		fmt.Println(err.Err)
		if err.Err.Error() == "bind: address already in use" {
			fmt.Println("interrupting other daemon remotely...")
			con, err := net.Dial("unix", sockfile)
			if err != nil {
				fmt.Println("failed to quit other daemon")
				os.Remove(sockfile)
				return net.Listen("unix", sockfile)
			}
			con.Write([]byte{byte(quitMsg)})
			con.Close()
			fmt.Println("sent quit message")
			time.Sleep(time.Millisecond * 250)
			return net.Listen("unix", sockfile)
		}
	default:
	}
	return l, err
}

func handleClient(fd net.Conn, locker Locker) {
	command := make([]byte, 1)
	_, err := fd.Read(command)
	if err != nil {
		notify(err.Error())
		return
	}
	switch command[0] {
	case lockMsg:
		go func() {
			fmt.Println("locking due to unixsock", fd)
			locker.Lock()
		}()
	case suspendMsg:
		go func() {
			fmt.Println("suspending due to unixsock", fd)
			Suspend()
		}()
	case quitMsg:
		signalChan <- os.Interrupt
	case invalidMsg:
	default:
		notify("invalid message!")
	}
}

func notify(msg string) {
	fmt.Println(msg)
	exec.Command("notify-send", msg).Run()
}
