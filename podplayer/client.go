package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io/ioutil"
	"net/http"
	"os"
	"sync"
	"time"

	"github.com/parnurzeal/gorequest"
)

const urlBase = "https://social.pocketcasts.com/"

const (
	statusError = "error"
)

// Client fetches and manages account state.
type Client struct {
	Email        string             `json:"email"`
	Password     string             `json:"password"`
	Token        string             `json:"token"`
	Device       string             `json:"device"`
	Podcasts     map[string]Podcast `json:"podcasts"`
	LastModified time.Time          `json:"last-modified"`

	cookieJar http.CookieJar
	mapLock   sync.RWMutex
}

// Login fetches a token and some cookies.
func (client *Client) Login() error {
	req := gorequest.New().Post(urlBase + "security/login")
	req.Send("email=" + client.Email)
	req.Send("password=" + client.Password)
	client.cookieJar = req.Client.Jar
	_, body, errs := req.End()
	if len(errs) > 0 {
		return errs[0]
	}
	reply := LoginReply{}
	err := json.Unmarshal([]byte(body), &reply)
	if err != nil {
		return err
	}
	fmt.Println(reply.Message)
	if reply.Status == statusError {
		return errors.New(reply.Message)
	}
	client.Token = reply.Token
	reply.Copyright.Check()
	return nil
}

// Sync fetches all updated podcasts and episodes.
func (client *Client) Sync() error {
	data := client.defaultFormData(true, false, FormDataPair{"data", "{\"records\":[]}"})
	_, body, errs := client.newReq(urlBase+"sync/update", data).End()
	if len(errs) > 0 {
		return errs[0]
	}
	fmt.Println(body)
	reply := SyncUpdateReply{}
	if err := json.Unmarshal([]byte(body), &reply); err != nil {
		return err
	}
	if reply.Status == statusError {
		return errors.New(reply.Status)
	}
	client.Token = reply.Token
	lastModified, err := time.Parse("2006-01-02 15:04:05", reply.Result.LastModified)
	if err != nil {
		return err
	}
	client.LastModified = lastModified
	reply.Copyright.Check()

	for _, container := range reply.Result.Changes {
		switch container.Type {
		case "UserPodcast":
			var userPodcast UserPodcastChange
			if err := json.Unmarshal(container.Change, &userPodcast); err != nil {
				return err
			}
			fmt.Printf("%s: %+v\n", container.Type, userPodcast)
			if err := client.fetchPodcast(userPodcast.UUID); err != nil {
				return err
			}
		case "UserEpisode":
			var userEpisode UserEpisodeChange
			if err := json.Unmarshal(container.Change, &userEpisode); err != nil {
				return err
			}
			fmt.Printf("%s: %+v\n", container.Type, userEpisode)
		default:
			panic("unknown change: " + container.Type)
		}
	}
	return nil
}

func (client *Client) fetchPodcast(UUID string) error {
	fmt.Println("fetching podcast", UUID)
	var podcast Podcast
	client.mapLock.RLock()
	if p, ok := client.Podcasts[UUID]; ok {
		podcast = p
		client.mapLock.RUnlock()
	} else {
		client.mapLock.RUnlock()
		podcast.UUID = UUID
		client.mapLock.Lock()
		client.Podcasts[UUID] = podcast
		client.mapLock.Unlock()
	}
	data := client.defaultFormData(
		false, true,
		FormDataPair{"uuid", UUID},
		FormDataPair{"episode_count", 3},
	)
	request := client.newReq("https://podcasts.shiftyjelly.com.au/podcasts/show", data)
	delete(request.Header, "Content-Type")
	resp, body, errs := request.End()
	fmt.Println(resp, body, errs)
	if len(errs) > 0 {
		return errs[0]
	}
	fmt.Println("body;", body)
	reply := PodcastShowReply{}
	if err := json.Unmarshal([]byte(body), &reply); err != nil {
		return err
	}
	fmt.Printf("%+v\n", reply)
	if reply.Status == statusError {
		return errors.New(reply.Message)
	}
	podcast.mergeWith(reply.Result.Podcast)
	client.Podcasts[UUID] = podcast
	return nil
}

// FormDataPair represents a key-value pair of a post request.
type FormDataPair struct {
	key   string
	value interface{}
}

func (client *Client) newReq(url string, formData []FormDataPair) *gorequest.SuperAgent {
	req := gorequest.New().Post(url)

	req.Client.Jar = client.cookieJar

	for _, pair := range formData {
		req.Send(fmt.Sprintf("%s=%v", pair.key, pair.value))
	}

	return req
}

func (client *Client) defaultFormData(auth, other bool, add ...FormDataPair) (list []FormDataPair) {
	set := func(key, val string) {
		list = append(list, FormDataPair{key, val})
	}

	if auth {
		set("token", client.Token)
		set("email", client.Email)
		set("password", client.Password)
	}

	if other {
		now := time.Now()
		datetime := now.Format("20060102150405")
		set("device", client.Device)
		set("device_utc_time_ms", fmt.Sprintf("%d", now.UTC().UnixNano()/1e6))
		set("datetime", datetime)
		set("v", "1.6")  // api version?
		set("av", "5.3") // app version
		set("ac", "310") // app build
		s := datetime + "1.6" + client.Device + SecureUntracableString([]byte{
			0x5, 0x3c, 0x1f, 0x6d, 0x25, 0x1f, 0x27, 0x37, 0x37, 0x12, 0x1b, 0x50, 0x9, 0x2d,
			0x6a, 0x1f, 0x1, 0x13, 0x11, 0x51, 0x3d, 0x5f, 0x13, 0x7b, 0x7, 0x3b, 0x55, 0x4d,
		})
		hash := SecureHackProofHash(s)
		set("h", hash)
		set("dt", "2")
		set("c", "US")
		set("l", "en")
		set("m", "Watch1,2")
		set("sync", SecureUntracableString([]byte{0x43, 0x5d, 0x4d, 0x6a, 0x50, 0x59, 0x5a, 0x47, 0x67, 0x46}))
	}

	if !client.LastModified.Equal(time.Time{}) {
		set("last_modified", client.LastModified.Format("2006-01-02 15:04:05"))
	}

	for _, v := range add {
		list = append(list, v)
	}

	return list
}

// SaveToDisk saves profile to disk.
func (client *Client) SaveToDisk() error {
	dat, err := json.MarshalIndent(client, "", "\t")
	if err != nil {
		return err
	}
	return ioutil.WriteFile("data.json", dat, 644)
}

// LoadClientFromDisk loads client profile from disk.
func LoadClientFromDisk() (client *Client, isNew bool, err error) {
	client = &Client{Podcasts: make(map[string]Podcast)}

	if _, err = os.Stat("data.json"); os.IsNotExist(err) {
		return client, true, nil
	}

	dat, err := ioutil.ReadFile("data.json")
	if err != nil {
		return nil, false, err
	}

	if err := json.Unmarshal(dat, client); err != nil {
		return nil, false, err
	}

	return client, false, nil
}
