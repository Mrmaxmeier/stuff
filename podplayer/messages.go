package main

import (
	"encoding/json"
	"fmt"
)

type PlayingStatus int

const (
	Uninitialized PlayingStatus = iota
	NotPlayed
	InProgress
	Finished
)

type Copyright string

// Check checks if the copyright string matches
func (copyright Copyright) Check() {
	if copyright != "Shifty Jelly - Pocket Casts" {
		fmt.Println(copyright)
		panic("copyright doesn't match...")
	}
}

type LoginReply struct {
	Status    string      `json:"status"`
	Token     string      `json:"token"`
	Copyright Copyright   `json:"copyright"`
	Result    interface{} `json:"result"`
	Message   string      `json:"message"`
}

type UserEpisodeChange struct {
	UUID          string        `json:"uuid"`
	PlayingStatus PlayingStatus `json:"playing_status"`
	PlayedUpTo    int           `json:"played_up_to"`
	IsDeleted     bool          `json:"is_deleted"`
	Duration      int           `json:"duration"`
	Starred       bool          `json:"starred"`
}

type UserPodcastChange struct {
	UUID              string      `json:"uuid"`
	IsDeleted         bool        `json:"is_deleted"`
	Subscribed        bool        `json:"subscribed"`
	AutoStartFrom     int         `json:"auto_start_from"`
	EpisodesSortOrder interface{} `json:"episodes_sort_order"`
}

type SyncUpdateReply struct {
	Status    string    `json:"status"`
	Token     string    `json:"token"`
	Copyright Copyright `json:"copyright"`
	Result    struct {
		Changes []struct {
			Type   string          `json:"type"`
			Change json.RawMessage `json:"fields"`
		} `json:"changes"`
		LastModified string `json:"last_modified"`
	} `json:"result"`
}

type PodcastShowReply struct {
	Status  string `json:"status"`
	Message string `json:"message"`
	Result  struct {
		Podcast Podcast `json:"podcast"`
	} `json:"result"`
}

type EpisodeChangeUpdate struct {
	UUID                  string        `json:"uuid"`
	PlayingStatus         PlayingStatus `json:"playing_status"`
	PlayingStatusModified int64         `json:"playing_status_modified"`
	PlayedUpTo            float64       `json:"played_up_to"`
	PlayedUpToModified    int64         `json:"played_up_to_modified"`
	UserPodcastUUID       string        `json:"user_podcast_uuid"`
}

type SyncUpdateMessage struct {
	Fields interface{} `json:"fields"`
	Type   string      `json:"type"`
}

type SyncUpdateMessages struct {
	Records []SyncUpdateMessage `json:"records"`
}
