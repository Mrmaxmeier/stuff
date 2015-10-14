#!/usr/bin/python3

import socketserver
import time
import socket

def gen(string, padding):
	def f(balance):
		s = " " * balance + string + " " * (padding - balance) + "\n"
		return s, int(abs(padding/2 - balance) / padding * 3 + 1)
	while 1:
		for i in range(padding):
			yield f(i)
		for i in range(padding):
			yield f(padding - i)

class TCPHandler(socketserver.BaseRequestHandler):
	def handle(self):
		print("[+]", self.client_address)
		try:
			for string, count in gen("snaek", 8):
				for i in range(count):
					self.request.sendall(bytes(string, "utf-8"))
					time.sleep(0.05)
		except socket.error as e:
			if e.errno == 32:
				print("[-]", self.client_address)
			else:
				raise e

if __name__ == "__main__":
	HOST, PORT = "0.0.0.0", 17
	server = socketserver.TCPServer((HOST, PORT), TCPHandler)
	server.serve_forever()
