package main

import "github.com/bgentry/speakeasy"

func main() {
	var client Client
	var err error
	client.podcasts = make(map[string]*Podcast)
	if client.email == "" {
		client.email, err = speakeasy.Ask("email> ")
		if err != nil {
			panic(err)
		}
	}
	if client.password == "" {
		client.password, err = speakeasy.Ask("pass> ")
		if err != nil {
			panic(err)
		}
	}
	if e := client.Login(); e != nil {
		panic(e)
	}
	if e := client.Sync(); e != nil {
		panic(e)
	}
}
