#!/usr/bin/python3

import transmissionrpc
from bs4 import BeautifulSoup
import requests

TRACKERS = [
	"udp://tracker.leechers-paradise.org:6969",
	"udp://tracker.internetwarriors.net:1337",
	"udp://tracker.openbittorrent.com:80",
	"udp://tracker.coppersurfer.tk:6969",
	"udp://tracker.sktorrent.net:6969",
	"udp://exodus.desync.com:6969",
	"udp://open.demonii.com:1337"
]

print('connecting...')
tc = transmissionrpc.Client('localhost', port=9091)

print('\nlisting torrents...')

for torrent in tc.get_torrents():
	print()
	print('name:', torrent.name)
	print('status:', torrent.status)
	if torrent.status == 'seeding':
		print('ratio:', torrent.ratio)

print('\nchecking trackers ...')

def hasTracker(torrent, url):
	for tracker in torrent.trackers:
		if url in tracker['announce'] or url in tracker['scrape']:
			return True

def update_trackers(torrent):
	for url in TRACKERS:
		if not hasTracker(torrent, url):
			tc.change_torrent(torrent.id, trackerAdd=[url])
			print('adding tracker', url, 'to', torrent.name)
			torrent.update()

for torrent in tc.get_torrents():
	update_trackers(torrent)

def get_torrent(name):
	sanetized = lambda s: s.lower().replace('-', ' ').replace('.', ' ').replace(' iso', '').replace('_', ' ')
	for t in tc.get_torrents():
		if t.name == name or sanetized(t.name) == sanetized(name):
			return t
	return None

print('scraping linuxtracker.org')
html = requests.get('http://linuxtracker.org/index.php').text
soup = BeautifulSoup(html, 'html.parser')

def confirm(text):
	while True:
		i = input(text + ' [Y/n] ')
		if i in ['Y', 'y', '']:
			return True
		elif i in ['N', 'n']:
			return False

block = soup.find('td', id='mcol')
for tlist in block.find_all('div', class_='block'):
	print('  ->', tlist.find('div', class_='block-head').find('div', class_='block-head-title').string)
	torrents = tlist.find_all('tr')
	for torrent in torrents[1:]:
		tds = torrent.find_all('td')
		dl = 'http://linuxtracker.org/' + tds[0].find('a')['href']
		name = tds[1].find('a').string
		size = tds[5].string
		seeds = int(tds[6].string)
		leechs = int(tds[7].string)
		if seeds < 5 or leechs < 15:
			continue
		slratio = seeds / leechs
		if slratio > 20:
			continue
		print('    -', repr(name), size, seeds, leechs, round(slratio, 2), end=' ')
		if get_torrent(name):
			print('✓')
			continue
		print()
		if not confirm('    add ' + repr(name) + '?'):
			print('    -', repr(name), size, seeds, leechs, round(slratio, 2), '✗')
			continue
		dlhtml = requests.get(dl).text
		dlsoup = BeautifulSoup(dlhtml, 'html.parser')
		dlurl = 'http://linuxtracker.org/' + dlsoup.find_all('a')[-4]['href'] # TODO remove magic -4
		tc.add_torrent(dlurl)
		if get_torrent(name):
			update_trackers(get_torrent(name))
		else:
			print('WARNING: name doesnt match')
		print('    -', repr(name), size, seeds, leechs, round(slratio, 2), '✓')
