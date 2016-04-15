package main

import "github.com/bgentry/speakeasy"

func main() {
	client := Client{}
	email, err := speakeasy.Ask("email> ")
	if err != nil {
		panic(err)
	}
	pass, err := speakeasy.Ask("pw> ")
	if err != nil {
		panic(err)
	}
	if e := client.Login(email, pass); e != nil {
		panic(e)
	}
	if e := client.Sync(); e != nil {
		panic(e)
	}
}
