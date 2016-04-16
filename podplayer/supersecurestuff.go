package main

import (
	"bytes"
	"crypto/sha1"
	"encoding/binary"

	"golang.org/x/text/encoding/charmap"
)

const leetMessage = "plz_dont_sue:^)"

func SecureUntracableString(s []byte) string {
	for i := 0; i < len(s); i++ {
		s[i] = s[i] ^ leetMessage[i%len(leetMessage)]
	}
	return string(s)
}

func int8FromUint8Bits(in uint8) (out int8) {
	buf := new(bytes.Buffer)
	binary.Write(buf, binary.LittleEndian, in)
	binary.Read(buf, binary.LittleEndian, &out)
	return out
}

func SecureHackProofHash(s string) string {
	sb, _ := charmap.ISO8859_1.NewEncoder().Bytes([]byte(s))
	udigest := sha1.Sum(sb)
	digest := make([]int8, len(udigest))
	for i := 0; i < len(digest); i++ {
		digest[i] = int8FromUint8Bits(udigest[i])
	}
	var output []byte
	for i := 0; i < len(digest); i++ {
		i2 := int(uint(digest[i])>>4) & 15
		i3 := 0
		for {
			if i2 < 0 || i2 > 9 {
				output = append(output, byte(i2+87))
			} else {
				output = append(output, byte(i2+48))
			}
			i4 := int(digest[i]) & 15
			i2 = i3 + 1
			if i3 > 0 {
				break
			}
			i3 = i2
			i2 = i4
		}
	}
	return string(output)
}
