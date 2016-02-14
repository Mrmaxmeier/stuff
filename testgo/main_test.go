package main

import (
	"fmt"
	"testing"
	"time"
)

func TestBlah(t *testing.T) {
	fmt.Println("TestBlahOut\nMultiline\nOutput")
}

func TestBlah_2(t *testing.T) {}

func TestFail(t *testing.T) {
	t.Error("failed\nhiu\nhuhu")
}

func TestBlahHeavy(t *testing.T) {
	time.Sleep(time.Second)
}

func TestBlahHeavyOutput(t *testing.T) {
	for i := 0; i < 5; i++ {
		fmt.Println(i)
		time.Sleep(time.Millisecond * 250)
	}
}

func BenchmarkHello(b *testing.B) {
	for i := 0; i < b.N; i++ {
		fmt.Sprintf("%d%v%s", 1, 1, "a")
	}
}

func Benchmark2(b *testing.B) {
	for i := 0; i < b.N; i++ {
		time.Sleep(time.Millisecond)
	}
}
