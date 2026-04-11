pub const fn splitmix64(mut x: u64) -> u64 {
	x = x.wrapping_add(0x9E3779B97F4A7C15);
	x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
	x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
	x ^ (x >> 31)
}
pub const fn fibo_hash_64(x: u64) -> u64 {
	const K: u64 = 11400714819323198485;
	let h = x.wrapping_mul(K);
	//h.swap_bytes()
	h ^ (h >> 32)
}
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use std::{
	cell::{Ref, RefCell}, io::Write, time::{Duration, Instant}
};

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[macro_export]
macro_rules! show_mem {
	($title:expr, $data:expr) => {
		#[allow(clippy::size_of_ref)]
		let size = std::mem::size_of_val(&$data);
		let plural = if size > 1 { "s" } else { " " };
		let address = std::ptr::addr_of!($data);
		println!(
			"{}: {:2} byte{} @ {:?} ~~> {:?}",
			$title, size, plural, address, $data,
		);
	};
}
pub use show_mem;
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Debug, Default, Clone)]
pub struct Rng {
	seed: u64,
}

impl Rng {
	pub fn new() -> Self {
		#[cfg(target_arch = "wasm32")]
		let now = web_sys::js_sys::Date::now() as u32;
		#[cfg(not(target_arch = "wasm32"))]
		let now = (std::time::UNIX_EPOCH.elapsed().unwrap().as_millis() >> 8) as u32;

		let addr = std::ptr::addr_of!(now) as usize as u32;
		Self::from_seed(((now as u64) << 32) | addr as u64)
	}

	pub const fn from_seed(seed: u64) -> Self {
		Self { seed }
	}

	pub const fn seed(&self) -> u64 {
		self.seed
	}
	pub const fn u8(&mut self) -> u8 {
		self.advance() as u8
	}
	pub const fn u16(&mut self) -> u16 {
		self.advance() as u16
	}
	pub const fn u32(&mut self) -> u32 {
		self.advance()
	}
	pub const fn u64(&mut self) -> u64 {
		((self.advance() as u64) << 32) | self.advance() as u64
	}
	pub const fn u128(&mut self) -> u128 {
		((self.u64() as u128) << 64) | self.u64() as u128
	}
	pub fn number<T: RngNumber>(&mut self) -> T {
		T::number(self)
	}

	pub fn sign<T: RngSign>(&mut self) -> T {
		T::sign(self)
	}

	pub fn range<T: RngRange>(
		&mut self,
		low: T,
		high: T,
	) -> T {
		T::range(self, low, high)
	}

	// random value with mean 0.0 and standard deviation 1.0
	pub fn normal<T: RngNormal>(&mut self) -> T {
		T::normal(self)
	}

	pub fn shuffle<T>(
		&mut self,
		data: &mut [T],
	) {
		let length = data.len();
		for dst in 0..length {
			let src = self.range(dst, length);
			data.swap(dst, src);
		}
	}

	// Adapted from MWC generator described in George Marsaglia's post
	// ``Random numbers for C: End, at last?'' on sci.stat.math,
	// sci.math, sci.math.num-analysis, sci.crypt, sci.physics.research,
	// comp.os.msdos.djgpp (Thu, 21 Jan 1999 03:08:52 GMT)
	const fn advance(&mut self) -> u32 {
		const MASK_32: u64 = (1 << 32) - 1;
		const MASK_16: u64 = (1 << 16) - 1;
		let mut s0 = self.seed >> 32;
		let mut s1 = self.seed & MASK_32;
		s0 = (36969 * (s0 & MASK_16) + (s0 >> 16)) & MASK_32;
		s1 = (18000 * (s1 & MASK_16) + (s1 >> 16)) & MASK_32;
		self.seed = (s0 << 32) | s1;
		((s0 << 16) + s1) as u32
	}
}

pub trait RngNumber {
	fn number(rng: &mut Rng) -> Self;
}

impl RngNumber for bool {
	fn number(rng: &mut Rng) -> Self {
		rng.advance() & 1 != 0
	}
}

macro_rules! impl_rng_number_small {
	($( $int:ident ),+) => {$(
		impl RngNumber for $int {
			fn number(rng: &mut Rng) -> Self {
				rng.advance() as Self
			}
		}
	)+};
}
impl_rng_number_small! { i8, u8, i16, u16, i32, u32 }

impl RngNumber for u64 {
	fn number(rng: &mut Rng) -> Self {
		((rng.advance() as Self) << 32) | rng.advance() as Self
	}
}

impl RngNumber for u128 {
	fn number(rng: &mut Rng) -> Self {
		((u64::number(rng) as Self) << 64) | u64::number(rng) as Self
	}
}

impl RngNumber for usize {
	fn number(rng: &mut Rng) -> Self {
		match core::mem::size_of::<Self>() {
			1 => u8::number(rng) as Self,
			2 => u16::number(rng) as Self,
			4 => u32::number(rng) as Self,
			8 => u64::number(rng) as Self,
			16 => u128::number(rng) as Self,
			_ => unreachable!(),
		}
	}
}

macro_rules! impl_rng_number_large_signed {
	($( $s_int:ident | $u_int:ident ),+) => {$(
		impl RngNumber for $s_int {
			fn number(rng: &mut Rng) -> Self {
				$u_int::number(rng) as Self
			}
		}
	)+};
}
impl_rng_number_large_signed! { i64|u64, i128|u128, isize|usize }

macro_rules! impl_rng_number_real {
	($( $real:ident | $u_int:ident ),+) => {$(
		impl RngNumber for $real {
			// nb: 32 bits will overflow a 24-bit significand for SP-IEEE754
			// nb: 64 bits will overflow a 53-bit significand for DP-IEEE754
			fn number(rng: &mut Rng) -> Self {
				const BITS: $u_int = $real::MANTISSA_DIGITS as $u_int;
				const MASK: $u_int = (1 << BITS) - 1;
				const DIV: $real = 1.0 / (MASK + 1) as $real;
				($u_int::number(rng) & MASK) as Self * DIV
			}
		}
	)+};
}
impl_rng_number_real! { f32|u32, f64|u64 }

pub trait RngSign {
	fn sign(rng: &mut Rng) -> Self;
}

macro_rules! impl_rng_sign {
	($( $s_type:ident ),+) => {$(
		impl RngSign for $s_type {
			fn sign(rng: &mut Rng) -> Self {
				if bool::number(rng) {
					1 as Self
				} else {
					-1 as Self
				}
			}
		}
	)+};
}
impl_rng_sign! { i8, i16, i32, i64, i128, isize, f32, f64 }

pub trait RngRange {
	fn range(
		rng: &mut Rng,
		low: Self,
		high: Self,
	) -> Self;
}

macro_rules! impl_rng_range_int {
	($( $u_int:ident | $s_int:ident ),+) => {$(
		impl RngRange for $u_int {
			fn range(
				rng: &mut Rng,
				low: Self,
				high: Self,
			) -> Self {
				low + rng.number::<Self>() % (high - low)
			}
		}
		impl RngRange for $s_int {
			fn range(
				rng: &mut Rng,
				low: Self,
				high: Self,
			) -> Self {
				type U = $u_int;
				let (lu, hu) = (low as U, high as U);
				U::range(rng, 0, hu.wrapping_sub(lu)).wrapping_add(lu)
					as Self
			}
		}
	)+};
}
impl_rng_range_int! {
	u8|i8, u16|i16, u32|i32, u64|i64, u128|i128, usize|isize
}

macro_rules! impl_rng_range_real {
	($( $real:ident ),+) => {$(
		impl RngRange for $real {
			fn range(
				rng: &mut Rng,
				low: Self,
				high: Self,
			) -> Self {
				low + rng.number::<Self>() * (high - low)
			}
		}
	)+};
}
impl_rng_range_real! { f32, f64 }

pub trait RngNormal {
	fn normal(rng: &mut Rng) -> Self;
}

macro_rules! impl_rng_normal {
	($( $real:ident ),+) => {$(
		impl RngNormal for $real {
			fn normal(rng: &mut Rng) -> Self {
				let r1 = core::$real::consts::PI * 2.0 * rng.number::<Self>();
				let r2 = 1.0 - rng.number::<Self>();
				r1.cos() * (-2.0 * r2.ln()).sqrt() // Box-Muller method
			}
		}
	)+};
}
impl_rng_normal! { f32, f64 }

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[macro_export]
macro_rules! bm_call {
	( $($fnct:ident)::+ ( $($arg:expr),* ) ) => {
		core::hint::black_box( $($fnct)::+ (
			$(core::hint::black_box($arg)),* ) )
	};
	( $($fnct:ident).+ ( $($arg:expr),* ) ) => {
		core::hint::black_box( $($fnct).+ (
			$(core::hint::black_box($arg)),* ) )
	};
	( $($fnct:ident::)+ < $($type:ty),+ > ( $($arg:expr),* ) ) => {
		core::hint::black_box( $($fnct::)+ < $($type),+ > (
			$(core::hint::black_box($arg)),* ) )
	};
	( $($fnct:ident).+ :: < $($type:ty),+ > ( $($arg:expr),* ) ) => {
		core::hint::black_box( $($fnct).+ :: < $($type),+ > (
			$(core::hint::black_box($arg)),* ) )
	};
	( $t:expr, $($fnct:ident)::+ ( $($arg:expr),* ) ) => {
		$t.add(|| s4::bm_call!($($fnct)::+ ( $($arg),* )))
	};
	( $t:expr, $($fnct:ident).+ ( $($arg:expr),* ) ) => {
		$t.add(|| s4::bm_call!($($fnct).+ ( $($arg),* )))
	};
	( $t:expr, $($fnct:ident::)+ < $($type:ty),+ > ( $($arg:expr),* ) ) => {
		$t.add(|| s4::bm_call!($($fnct::)+ < $($type),+ > ( $($arg),* )))
	};
	( $t:expr, $($fnct:ident).+ :: < $($type:ty),+ > ( $($arg:expr),* ) ) => {
		$t.add(|| s4::bm_call!($($fnct).+ :: < $($type),+ > ( $($arg),* )))
	};
}
pub use bm_call;

#[derive(Debug, Clone)]
pub struct BenchmarkTimer {
	elapsed: Duration,
}

impl BenchmarkTimer {
	pub fn add<T>(
		&mut self,
		fnct: impl FnOnce() -> T,
	) -> T {
		let t0 = Instant::now();
		let r = fnct();
		self.elapsed += t0.elapsed();
		r
	}
}

#[derive(Debug, Clone)]
pub struct Benchmark {
	title: String,
	samples: RefCell<Vec<(f64, f64)>>,
	records: RefCell<Vec<BenchmarkRecord>>,
}

impl Benchmark {
	pub fn new(title: impl Into<String>) -> Self {
		if cfg!(debug_assertions) {
			eprintln!(
				"!!! benchmarking in DEBUG mode is totally irrelevant !!!"
			);
		}
		Self {
			title: title.into(),
			samples: RefCell::new(Vec::with_capacity(200)),
			records: RefCell::new(Vec::new()),
		}
	}

	pub fn record(
		&self,
		title: impl Into<String>,
		minimal_seconds: f64,
		mut fnct: impl FnMut(&mut BenchmarkTimer),
	) {
		let title = title.into();
		if self.records.borrow().iter().any(|r| r.title == title) {
			panic!(
				"multiple records with title {:?} in benchmark {:?}",
				title, self.title
			);
		}
		let running_limit = minimal_seconds.max(1.0);
		let warming_limit = 0.2_f64.max(running_limit / 5.0);
		let mut warming_duration = 0.0;
		let mut running_duration = 0.0;
		let mut iterations = 0;
		let mut samples = self.samples.borrow_mut();
		samples.clear();
		while running_duration < running_limit {
			let repeat = 1.1_f64.powi(samples.len() as i32).round() as usize;
			let mut timer = BenchmarkTimer {
				elapsed: Duration::ZERO,
			};
			for _ in 0..repeat {
				fnct(&mut timer);
			}
			if timer.elapsed.is_zero() {
				panic!("no time spent in benchmark");
			}
			if warming_duration < warming_limit {
				warming_duration += timer.elapsed.as_secs_f64();
			} else {
				iterations += repeat;
				let elapsed = timer.elapsed.as_secs_f64();
				running_duration += elapsed;
				samples.push((repeat as f64, elapsed));
			}
		}
		let record = BenchmarkRecord::new(title, iterations, &mut samples);
		let mut records = self.records.borrow_mut();
		records.push(record);
		let best_avg = records
			.iter()
			.map(|r| r.avg_duration)
			.min()
			.unwrap()
			.as_secs_f64();
		for record in records.iter_mut() {
			record.performance = (best_avg > f64::EPSILON)
				.then(|| best_avg / record.avg_duration.as_secs_f64());
		}
	}

	pub fn title(&self) -> &str {
		&self.title
	}

	pub fn records(&self) -> Ref<'_, Vec<BenchmarkRecord>> {
		self.records.borrow()
	}
}

impl std::fmt::Display for Benchmark {
	fn fmt(
		&self,
		f: &mut std::fmt::Formatter<'_>,
	) -> std::fmt::Result {
		let records = self.records.borrow();
		if records.is_empty() {
			writeln!(f, "{}: no record", self.title)
		} else {
			let show_perf = records.len() > 1
				&& records.iter().all(|r| r.performance.is_some());
			writeln!(f, "{}:", self.title)?;
			let title_w = records
				.iter()
				.map(|r| r.title.chars().count())
				.max()
				.unwrap_or(0);
			let avg_texts = Vec::from_iter(
				records.iter().map(|r| format!("{:.3?}", r.avg_duration)),
			);
			let avg_w = avg_texts
				.iter()
				.map(|a| a.chars().count())
				.max()
				.unwrap_or(0);
			for (record, avg) in records.iter().zip(avg_texts) {
				let perf = if show_perf {
					format!("{:5.1}% ", 100.0 * record.performance.unwrap())
				} else {
					String::new()
				};
				writeln!(
					f,
					"• {}“{}”{} {}{} ({})",
					perf,
					record.title,
					" ".repeat(title_w - record.title.chars().count()),
					avg,
					" ".repeat(avg_w - avg.chars().count()),
					record.remarks(),
				)?;
			}
			Ok(())
		}
	}
}

fn filter_outliers(samples: &mut Vec<(f64, f64)>) -> usize {
	let n = samples.len();
	if n > 0 {
		// https://en.wikipedia.org/wiki/Outlier#Tukey's_fences
		// https://en.wikipedia.org/wiki/Quartile#Method_4
		samples.sort_unstable_by(|&(r1, e1), &(r2, e2)| {
			(e1 * r2).partial_cmp(&(e2 * r1)).unwrap()
		});
		let make_q = |mut q: f64| {
			q *= n as f64;
			let (qi, qf) = (q as usize, q.fract());
			let (lr, le) = samples[qi.min(n - 1)];
			let (hr, he) = samples[(qi + 1).min(n - 1)];
			le / lr * (1.0 - qf) + he / hr * qf
		};
		let q1 = make_q(0.25);
		let q3 = make_q(0.75);
		let iqr = q3 - q1;
		let low = q1 - iqr * 1.5;
		let high = q3 + iqr * 1.5;
		samples.retain(|&(r, e)| e >= r * low && e <= r * high);
	}
	n - samples.len()
}

fn ordinary_least_squares(samples: &[(f64, f64)]) -> (f64, f64, f64) {
	// ordinary least squares and standard deviation
	let sx = samples.iter().map(|(r, _e)| r).sum::<f64>();
	let sy = samples.iter().map(|(_r, e)| e).sum::<f64>();
	let sxx = samples.iter().map(|(r, _e)| r * r).sum::<f64>();
	let syy = samples.iter().map(|(_r, e)| e * e).sum::<f64>();
	let sxy = samples.iter().map(|(r, e)| r * e).sum::<f64>();
	let n = samples.len() as f64;
	let covar = n * sxy - sx * sy;
	let xvar = n * sxx - sx * sx;
	let yvar = n * syy - sy * sy;
	if (xvar * yvar).abs() > f64::EPSILON {
		let slope = covar / xvar;
		let offset = (sy - slope * sx) / n;
		let r2 = (covar * covar) / (xvar * yvar);
		(slope, offset, r2)
	} else {
		(0.0, 0.0, 0.0)
	}
}

fn relative_deviation(samples: &[(f64, f64)]) -> (f64, f64) {
	let n = samples.len() as f64;
	if n > f64::EPSILON {
		let avg = samples.iter().map(|(r, e)| e / r).sum::<f64>() / n;
		let variance = samples
			.iter()
			.map(|(r, e)| (avg - e / r).powi(2))
			.sum::<f64>()
			/ n;
		(avg, variance.sqrt() / avg)
	} else {
		(0.0, 0.0)
	}
}

#[derive(Debug, Clone)]
pub struct BenchmarkRecord {
	pub title: String,
	pub avg_duration: Duration,
	pub goodness_of_fit: f64,
	pub relative_deviation: f64,
	pub outliers: usize,
	pub samples: usize,
	pub iterations: usize,
	pub performance: Option<f64>,
}

impl BenchmarkRecord {
	fn new(
		title: String,
		iterations: usize,
		samples: &mut Vec<(f64, f64)>,
	) -> Self {
		let outliers = filter_outliers(samples);
		let (slope, _offset, goodness_of_fit) =
			ordinary_least_squares(samples);
		let (avg, relative_deviation) = relative_deviation(samples);
		let avg_duration = Duration::from_secs_f64(if slope > f64::EPSILON {
			slope
		} else {
			avg // probably not significative
		});
		Self {
			title,
			avg_duration,
			goodness_of_fit,
			relative_deviation,
			outliers,
			samples: samples.len(),
			iterations,
			performance: None,
		}
	}

	fn remarks(&self) -> String {
		let r2_warning = if self.goodness_of_fit < 0.9 {
			format!(", R²={:.2}", self.goodness_of_fit)
		} else {
			String::new()
		};
		let ol_warning = if self.outliers * 100 >= self.samples * 5 {
			let plural = if self.outliers > 1 { "s" } else { "" };
			format!(", {} outlier{}", self.outliers, plural)
		} else {
			String::new()
		};
		let sm_warning = if self.samples < 50 {
			let plural = if self.samples > 1 { "s" } else { "" };
			format!(", only {} sample{}", self.samples, plural)
		} else {
			String::new()
		};
		format!(
			"σ={:.2}%{}{}{}",
			self.relative_deviation * 100.0,
			r2_warning,
			ol_warning,
			sm_warning
		)
	}
}

impl std::fmt::Display for BenchmarkRecord {
	fn fmt(
		&self,
		f: &mut std::fmt::Formatter<'_>,
	) -> std::fmt::Result {
		write!(
			f,
			"{}: {:.3?} ({})",
			self.title,
			self.avg_duration,
			self.remarks(),
		)
	}
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub struct MiniSvg {
	width: f64,
	height: f64,
	svg: Vec<u8>,
}

impl MiniSvg {
	pub fn new(
		width: f64,
		height: f64,
	) -> Self {
		let mut svg = Vec::new();
		write!(&mut svg, r#"<svg viewBox="0 0 {} {}""#, width, height)
			.unwrap();
		writeln!(&mut svg, r#" xmlns="http://www.w3.org/2000/svg">"#)
			.unwrap();
		Self { width, height, svg }
	}

	pub fn width(&self) -> f64 {
		self.width
	}

	pub fn height(&self) -> f64 {
		self.height
	}

	#[allow(clippy::too_many_arguments)]
	pub fn rect(
		&mut self,
		x: f64,
		y: f64,
		w: f64,
		h: f64,
		border: Option<f64>,
		color: Option<&str>,
		border_color: Option<&str>,
		title: Option<&str>,
	) {
		let svg = &mut self.svg;
		write!(
			svg,
			r#"<rect x="{}" y="{}" width="{}" height="{}""#,
			x, y, w, h
		)
		.unwrap();
		write!(
			svg,
			r#" fill="{}" stroke="{}" stroke-width="{}""#,
			color.unwrap_or("none"),
			border.map_or("none".to_owned(), |b| format!("{}", b)),
			border_color.unwrap_or("none"),
		)
		.unwrap();
		if let Some(title) = title {
			writeln!(svg, r#"><title>{}</title></rect>"#, title).unwrap();
		} else {
			writeln!(svg, r#"/>"#).unwrap();
		}
	}

	#[allow(clippy::too_many_arguments)]
	pub fn circle(
		&mut self,
		cx: f64,
		cy: f64,
		radius: f64,
		border: Option<f64>,
		color: Option<&str>,
		border_color: Option<&str>,
		title: Option<&str>,
	) {
		let svg = &mut self.svg;
		write!(svg, r#"<circle cx="{}" cy="{}" r="{}""#, cx, cy, radius)
			.unwrap();
		write!(
			svg,
			r#" fill="{}" stroke="{}" stroke-width="{}""#,
			color.unwrap_or("none"),
			border_color.unwrap_or("none"),
			border.map_or("none".to_owned(), |b| format!("{}", b)),
		)
		.unwrap();
		if let Some(title) = title {
			writeln!(svg, r#"><title>{}</title></circle>"#, title).unwrap();
		} else {
			writeln!(svg, r#"/>"#).unwrap();
		}
	}

	#[allow(clippy::too_many_arguments)]
	pub fn line(
		&mut self,
		x1: f64,
		y1: f64,
		x2: f64,
		y2: f64,
		width: f64,
		color: &str,
		title: Option<&str>,
	) {
		let svg = &mut self.svg;
		write!(
			svg,
			r#"<line x1="{}" y1="{}" x2="{}" y2="{}""#,
			x1, y1, x2, y2
		)
		.unwrap();
		write!(svg, r#" stroke="{}" stroke-width="{}""#, color, width)
			.unwrap();
		if let Some(title) = title {
			writeln!(svg, r#"><title>{}</title></line>"#, title).unwrap();
		} else {
			writeln!(svg, r#"/>"#).unwrap();
		}
	}

	pub fn into_code(mut self) -> String {
		writeln!(&mut self.svg, r#"</svg>"#).unwrap();
		String::from_utf8(self.svg).unwrap()
	}
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
