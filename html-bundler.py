#!/usr/bin/python3
from bs4 import BeautifulSoup
import click

def set_inner_html_from_file(element, attr):
	print('packing', element)
	path = element.get(attr)
	with open(path, 'r') as f:
		element.string = f.read()

def inline(element):
	if element.name == 'script':
		if element.get('src'):
			set_inner_html_from_file(element, 'src')
			del element['src']
	elif element.name == 'link':
		print(element)
		if element.get('rel') == ['stylesheet']:
			if element.get('href'):
				element.name = 'style'
				set_inner_html_from_file(element, 'href')
				del element['rel']
				del element['href']
	else:
		print('unhandled tag', element.name)
inline_tags = ['link', 'script']

@click.command()
@click.argument('input', type=click.File('r'))
@click.option('--output', default=None)
def main(input, output):
	input_file = input
	output = output or input_file.name.replace('.html', '.bundled.html')
	if output == input_file.name:
		print("won't overwrite file!")
		return

	soup = BeautifulSoup(input_file.read(), 'html.parser')
	for tag in inline_tags:
		for element in soup.find_all(tag):
			inline(element)

	print('writing', output)
	with open(output, 'w') as f:
		f.write(soup.prettify())


if __name__ == '__main__':
	main()
