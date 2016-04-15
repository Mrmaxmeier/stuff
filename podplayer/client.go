package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"

	"github.com/parnurzeal/gorequest"
)

const urlBase = "https://social.pocketcasts.com/"

// Client fetches and manages account state.
type Client struct {
	token     string
	email     string
	password  string
	cookieJar http.CookieJar
	podcasts  map[string]Podcast
	episodes  map[string]Episode
}

// Login fetches a token and some cookies.
func (client *Client) Login(email, password string) error {
	req := gorequest.New().Post(urlBase + "security/login")
	req.Send("email=" + email)
	req.Send("password=" + password)
	client.cookieJar = req.Client.Jar
	resp, body, errs := req.End()
	fmt.Println("errors", errs)
	if len(errs) > 0 {
		return errs[0]
	}
	fmt.Println("header:")
	fmt.Println(resp.Header)
	reply := LoginReply{}
	fmt.Println("body:")
	fmt.Println(body)
	err := json.Unmarshal([]byte(body), &reply)
	if err != nil {
		return err
	}
	fmt.Println(reply)
	if reply.Status == "error" {
		return errors.New(reply.Message)
	}
	client.token = reply.Token
	reply.Copyright.Check()
	client.email = email
	client.password = password
	return nil
}

// Sync fetches all updated podcasts and episodes.
func (client *Client) Sync() error {
	data := []FormDataPair{
		FormDataPair{"data", "{\"records\":[]}"},
	}
	resp, body, errs := client.newReg(urlBase+"sync/update", data).End()
	fmt.Println(resp, body, errs)
	if len(errs) > 0 {
		return errs[0]
	}
	fmt.Println(body)
	reply := SyncUpdateReply{}
	err := json.Unmarshal([]byte(body), &reply)
	if err != nil {
		return err
	}
	fmt.Println(reply)
	if reply.Status == "error" {
		return errors.New(reply.Status)
	}
	client.token = reply.Token
	reply.Copyright.Check()

	for _, container := range reply.Result.Changes {
		switch container.Type {
		case "UserPodcast":
			var userPodcast UserPodcastChange
			if err := json.Unmarshal(container.Change, &userPodcast); err != nil {
				return err
			}
			fmt.Printf("%s\n%+v\n", container.Type, userPodcast)
		case "UserEpisode":
			var userEpisode UserEpisodeChange
			if err := json.Unmarshal(container.Change, &userEpisode); err != nil {
				return err
			}
			fmt.Printf("%s\n%+v\n", container.Type, userEpisode)
		default:
			fmt.Println(container.Type)
			panic("unknown change")
		}
	}

	return nil
}

type FormDataPair struct {
	key   string
	value interface{}
}

func (client *Client) newReg(url string, formData []FormDataPair) *gorequest.SuperAgent {
	req := gorequest.New().Post(url)

	set := func(key string, val string) {
		req.Send(fmt.Sprintf("%s=%s", key, val))
	}
	set("token", client.token)
	set("email", client.email)
	set("password", client.password)

	/*
		  set("device_utc_time_ms", time.Now().UTC().UnixNano()/1e6)
			set("last_modified", time.Now().Format("2006-01-02 15:04:05"))
			set("datetime", time.Now().Format("20060102150405"))
			set("v", "1.6")
			set("av", "5.3") // app version
			set("ac", "310") // app build
			set("h", "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
			set("device", <UUID4>)
			set("dt", "2")
			set("c", "US")
			set("l", "en")
			set("m", "Device Model")
			set("sync", "xxxxxxxxxx")
	*/

	req.Client.Jar = client.cookieJar

	for _, pair := range formData {
		req.Send(fmt.Sprintf("%s=%v", pair.key, pair.value))
	}

	return req
}
