use std::ffi::{CStr, CString};
use std::str::from_utf8_unchecked;
use std::ops::Index;
use std::ptr;
use std::slice;
use std::mem;

use libc::{c_int, uint8_t};
use ffi::*;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Sample {
	None,

	U8(Type),
	I16(Type),
	I32(Type),
	F32(Type),
	F64(Type),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Type {
	Packed,
	Planar,
}

impl Sample {
	pub fn name(&self) -> &'static str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr(av_get_sample_fmt_name((*self).into())).to_bytes())
		}
	}

	pub fn packed(&self) -> Self {
		unsafe {
			Sample::from(av_get_packed_sample_fmt((*self).into()))
		}
	}

	pub fn planar(&self) -> Self {
		unsafe {
			Sample::from(av_get_planar_sample_fmt((*self).into()))
		}
	}

	pub fn is_planar(&self) -> bool {
		unsafe {
			av_sample_fmt_is_planar((*self).into()) == 1
		}
	}

	pub fn is_packed(&self) -> bool {
		!self.is_planar()
	}

	pub fn bytes(&self) -> usize {
		unsafe {
			av_get_bytes_per_sample((*self).into()) as usize
		}
	}

	pub fn buffer(&self, channels: u16, samples: usize, align: bool) -> Buffer {
		Buffer::new(*self, channels, samples, align)
	}
}

impl From<AVSampleFormat> for Sample {
	fn from(value: AVSampleFormat) -> Self {
		match value {
			AV_SAMPLE_FMT_NONE => Sample::None,

			AV_SAMPLE_FMT_U8  => Sample::U8(Type::Packed),
			AV_SAMPLE_FMT_S16 => Sample::I16(Type::Packed),
			AV_SAMPLE_FMT_S32 => Sample::I32(Type::Packed),
			AV_SAMPLE_FMT_FLT => Sample::F32(Type::Packed),
			AV_SAMPLE_FMT_DBL => Sample::F64(Type::Packed),

			AV_SAMPLE_FMT_U8P  => Sample::U8(Type::Planar),
			AV_SAMPLE_FMT_S16P => Sample::I16(Type::Planar),
			AV_SAMPLE_FMT_S32P => Sample::I32(Type::Planar),
			AV_SAMPLE_FMT_FLTP => Sample::F32(Type::Planar),
			AV_SAMPLE_FMT_DBLP => Sample::F64(Type::Planar),

			AV_SAMPLE_FMT_NB => Sample::None
		}
	}
}

impl From<&'static str> for Sample {
	fn from(value: &'static str) -> Self {
		unsafe {
			let value = CString::new(value).unwrap();

			Sample::from(av_get_sample_fmt(value.as_ptr()))
		}
	}
}

impl Into<AVSampleFormat> for Sample {
	fn into(self) -> AVSampleFormat {
		match self {
			Sample::None => AV_SAMPLE_FMT_NONE,

			Sample::U8(Type::Packed)  => AV_SAMPLE_FMT_U8,
			Sample::I16(Type::Packed) => AV_SAMPLE_FMT_S16,
			Sample::I32(Type::Packed) => AV_SAMPLE_FMT_S32,
			Sample::F32(Type::Packed) => AV_SAMPLE_FMT_FLT,
			Sample::F64(Type::Packed) => AV_SAMPLE_FMT_DBL,

			Sample::U8(Type::Planar)  => AV_SAMPLE_FMT_U8P,
			Sample::I16(Type::Planar) => AV_SAMPLE_FMT_S16P,
			Sample::I32(Type::Planar) => AV_SAMPLE_FMT_S32P,
			Sample::F32(Type::Planar) => AV_SAMPLE_FMT_FLTP,
			Sample::F64(Type::Planar) => AV_SAMPLE_FMT_DBLP,
		}
	}
}

pub struct Buffer {
	pub format: Sample,
	pub channels: u16,
	pub samples: usize,
	pub align: bool,

	buffer: *mut *mut uint8_t,
	size:   c_int,
}

impl Buffer {
	pub fn size(format: Sample, channels: u16, samples: usize, align: bool) -> usize {
		unsafe {
			av_samples_get_buffer_size(ptr::null_mut(), channels as c_int, samples as c_int, format.into(), !align as c_int) as usize
		}
	}

	pub fn new(format: Sample, channels: u16, samples: usize, align: bool) -> Self {
		unsafe {
			let mut buf = Buffer {
				format:   format,
				channels: channels,
				samples:  samples,
				align:    align,

				buffer: ptr::null_mut(),
				size:   0,
			};

			av_samples_alloc_array_and_samples(&mut buf.buffer, &mut buf.size,
			                 channels as c_int, samples as c_int,
			                 format.into(), !align as c_int);

			buf
		}
	}
}

impl Index<usize> for Buffer {
	type Output = [u8];

	fn index(&self, index: usize) -> &[u8] {
		if index >= self.samples {
			panic!("out of bounds");
		}

		unsafe {
			slice::from_raw_parts(*self.buffer.offset(index as isize), self.size as usize)
		}
	}
}

impl Clone for Buffer {
	fn clone(&self) -> Self {
		let mut buf = Buffer::new(self.format, self.channels, self.samples, self.align);
		buf.clone_from(self);

		buf
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_samples_copy(self.buffer, mem::transmute(source.buffer), 0, 0, source.samples as c_int, source.channels as c_int, source.format.into());
		}
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		unsafe {
			av_freep(mem::transmute(self.buffer));
		}
	}
}
