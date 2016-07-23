extern crate qrcode;
extern crate ansi_term;

use ansi_term::{Style, Color};
use qrcode::QrCode;

use std::iter::repeat;

#[derive(Debug)]
pub enum Padding {
	None,
	Some(usize),
}

pub fn render(qr: &QrCode, padding: Padding) {
	let data = qr.to_vec();
	let rows: Vec<&[bool]> = (0..qr.width()).map(|i| {
		&data[i * qr.width()..(i+1) * qr.width()]
	}).collect();
	let pad_rows: Vec<String> = match padding {
		Padding::None => vec![],
		Padding::Some(p) => {
			(0..p).map(|_| {
				repeat(" ").take(qr.width() + p * 2).collect()
			}).collect()
		},
	};
	for row in &pad_rows {
		println!("{}", row);
	}
	let outline_pad: String = match padding {
		Padding::None => "".into(),
		Padding::Some(p) => {
			repeat(" ").take(p * 2).collect()
		},
	};
	let outline_line: String = repeat("═").take(qr.width() * 2).collect();
	println!("{}╔{}╗{}", outline_pad, outline_line, outline_pad);
	for row in rows {
		let ansi: String = row.iter().map(|b| {
			let style = Style::new();
			if *b {
				style.on(Color::Black).paint("  ")
			} else {
				style.on(Color::White).paint("  ")
			}
		}).map(|s| {
			format!("{}", s)
		}).collect();
		let pad: String = match padding {
			Padding::None => "".into(),
			Padding::Some(p) => {
				repeat(" ").take(p * 2).collect()
			},
		};
		println!("{}║{}║{}", pad, ansi, pad);
	}
	println!("{}╚{}╝{}", outline_pad, outline_line, outline_pad);
	for row in &pad_rows {
		println!("{}", row);
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use qrcode::QrCode;

    #[test]
    fn it_works() {
		let string = "test qrcode data";
		let data: Vec<u8> = string.bytes().collect();
		let code = QrCode::new(data).unwrap();
		render(&code, Padding::Some(2));
		assert!(false);
    }
}
