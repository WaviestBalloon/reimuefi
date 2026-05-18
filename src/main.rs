#![no_main]
#![no_std]

extern crate alloc;

use alloc::{boxed::Box, vec};
use log::info;
use core::{error::Error, time::Duration};
use uefi::{prelude::*, println, proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput}};

static BMP_INDEX: [&[u8]; 17] = [
	include_bytes!("../frames/1.bmp"),
	include_bytes!("../frames/2.bmp"),
	include_bytes!("../frames/3.bmp"),
	include_bytes!("../frames/4.bmp"),
	include_bytes!("../frames/5.bmp"),
	include_bytes!("../frames/6.bmp"),
	include_bytes!("../frames/7.bmp"),
	include_bytes!("../frames/8.bmp"),
	include_bytes!("../frames/9.bmp"),
	include_bytes!("../frames/10.bmp"),
	include_bytes!("../frames/11.bmp"),
	include_bytes!("../frames/12.bmp"),
	include_bytes!("../frames/13.bmp"),
	include_bytes!("../frames/14.bmp"),
	include_bytes!("../frames/15.bmp"),
	include_bytes!("../frames/16.bmp"),
	include_bytes!("../frames/17.bmp")
];

static FRAME_SEQUENCE: [u32; 48] = [
	1, 2, 1, 2, 1, 3, 1, 2, 1, 2, 1, 4, 1, 2, 1, 2, 1, 2, 1, 5, 1, 2, 1, 2, 1, 6, 1, 7, 1, 8, 1, 9, 10, 11, 10, 12, 10, 12, 13, 1, 14, 15, 16, 17, 14, 15, 17, 14
];

pub struct BMPFrame {
	pub w: u32,
	pub h: u32,
	pub buffer: vec::Vec<BltPixel>
}

fn clamp_u8(input: u8, clamp_to: u8) -> u8 {
	if input > clamp_to {
		return clamp_to
	}

	input
}

pub fn bmp_to_buffer(bmp_bytes: &[u8]) -> Result<BMPFrame, Box<dyn Error>> {
	let mut buffer: vec::Vec<BltPixel> = vec![];
	let mut temp_x_buffer: vec::Vec<BltPixel> = vec![];

	let mut data_start_byte_position: u8 = 0;
	let mut pixel_position: i32 = -4; // have to shift 4 pixels for reason
	let mut w = 0;
	let mut h = 0;

	for (offset_index, byte) in bmp_bytes.iter().enumerate() {
		match offset_index {
			10 => {
				data_start_byte_position = *byte;
				info!("data_start_byte_position: {}", data_start_byte_position);
			},
			12 => {
				let raw_w: &[u8] = bmp_bytes.get(18..22).unwrap();
				w = u32::from_le_bytes(raw_w.try_into().unwrap());
				info!("width resolve: w: {}, h: {}", w, h);
			},
			16 => {
				let raw_h: &[u8] = bmp_bytes.get(22..26).unwrap();
				h = u32::from_le_bytes(raw_h.try_into().unwrap());
				info!("height resolve: w: {}, h: {}", w, h);
			},
			_ => { }
		}

		if offset_index >= data_start_byte_position.into() {
			if offset_index % 3 != 0 { // shift from B, G, R
				//info!("waiting for next start, offset_index: {} pixel_position: {}", offset_index, pixel_position);
				continue;
			}

			let r = byte;
			let g = bmp_bytes.get(offset_index + 1).unwrap_or(&0);
			let b = bmp_bytes.get(offset_index + 2).unwrap_or(&0);

			temp_x_buffer.push(BltPixel::new(clamp_u8(*b, 255), clamp_u8(*g, 255), clamp_u8(*r, 255)));
			pixel_position += 1;
			
			if pixel_position == w.try_into().unwrap() {
				temp_x_buffer.reverse();
				buffer.append(&mut temp_x_buffer);
				pixel_position = 0;
			}
		}
	}

	let expected_buffer_size: usize = (w * h).try_into()?;
	let buffer_length = buffer.len();

	if buffer_length < expected_buffer_size {
		let buffer_space = expected_buffer_size - buffer_length;
		for index in 0..buffer_space {
			if index % 2 != 0 {
				buffer.push(BltPixel::new(255, 0, 0));
			} else {
				buffer.push(BltPixel::new(255, 100, 100));
			}
		}
	}

	buffer.reverse();

	Ok(BMPFrame {
		w,
		h,
		buffer
	})
}

#[entry]
fn main() -> Status {
	uefi::helpers::init().unwrap();

	println!("Please enjoy reimu!");

	let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>().unwrap();
	let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();

	let (native_w, native_h) = gop.current_mode_info().resolution();

	loop {
		for index in FRAME_SEQUENCE {
			let actual_index = usize::try_from(index - 1).unwrap();
			let frame_data = BMP_INDEX.get(actual_index).unwrap();
			let buffer: BMPFrame = bmp_to_buffer(frame_data).unwrap();

			let buffer_w_usize: usize = buffer.w.try_into().unwrap(); // shhhh
			let buffer_h_usize: usize = buffer.h.try_into().unwrap();

			gop.blt(BltOp::BufferToVideo {
				buffer: &buffer.buffer,
				src: BltRegion::Full,
				dest: ((native_w / 2) - buffer_w_usize / 2, (native_h / 2) - buffer_h_usize / 2),
				dims: (buffer_w_usize, buffer_h_usize)
			}).unwrap();

			boot::stall(Duration::from_millis(200));
		}
	}
}
