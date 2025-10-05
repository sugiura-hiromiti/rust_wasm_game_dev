use crate::CanvasRenderingContext2d;
use crate::ContainerFixer;
use crate::HtmlImageElement;
use crate::JRslt;
use crate::LoadUtil;
use std::ops::Add;
use std::ops::Div;

fn draw_eq_sierpinski(ctx: &CanvasRenderingContext2d,) {
	let line_len = 300.0;
	let eq_tri_h = line_len * 3.0_f64.sqrt();

	let vert1 = (line_len, 0.0,).into();
	let vert2 = (0.0, eq_tri_h,).into();
	let vert3 = (line_len * 2.0, eq_tri_h,).into();

	ctx.triangle_empty(
		&(0.0, 0.0,).into(),
		&(0.0, 100.0,).into(),
		&(100.0, 0.0,).into(),
		Some(&(255, 255, 255,).into(),),
	);

	let mut fill_color = [0; 3];
	rand::fill(&mut fill_color,);
	let fill_color = (fill_color[0], fill_color[1], fill_color[2],).into();
	draw_sierpinski(ctx, &vert1, &vert2, &vert3, &fill_color, 6,);
}

fn draw_sierpinski(
	ctx: &CanvasRenderingContext2d,
	vert1: &Coord2d,
	vert2: &Coord2d,
	vert3: &Coord2d,
	fill_color: &Rgb,
	rec_count: usize,
) {
	if rec_count == 0 {
		return;
	}

	ctx.triangle_filled(vert1, vert2, vert3, None, fill_color,);

	let bw12 = (vert1 + vert2) / 2.0;
	let bw23 = (vert3 + vert2) / 2.0;
	let bw31 = (vert1 + vert3) / 2.0;

	let mut fill_color = [0; 3];
	rand::fill(&mut fill_color,);
	let fill_color = (fill_color[0], fill_color[1], fill_color[2],).into();
	draw_sierpinski(ctx, vert1, &bw12, &bw31, &fill_color, rec_count - 1,);
	draw_sierpinski(ctx, &bw12, vert2, &bw23, &fill_color, rec_count - 1,);
	draw_sierpinski(ctx, &bw23, &bw31, vert3, &fill_color, rec_count - 1,);
}

async fn draw_rhb(ctx: CanvasRenderingContext2d,) -> JRslt<(),> {
	let image = HtmlImageElement::new()?;
	image
		.load("Idle (1).png",)
		.await
		.consume_with(|_| {
			ctx.draw_image_with_html_image_element(&image, 0.0, 0.0,)
		},)
		.trans()?;
	Ok((),)
}
struct Coord2d {
	w: f64,
	h: f64,
}

impl Coord2d {
	pub fn new(w: f64, h: f64,) -> Self {
		Self { w, h, }
	}
}

impl Add for Coord2d {
	type Output = Self;

	fn add(self, rhs: Self,) -> Self::Output {
		Self { w: self.w + rhs.w, h: self.h + rhs.h, }
	}
}

impl Add for &Coord2d {
	type Output = Coord2d;

	fn add(self, rhs: Self,) -> Self::Output {
		(self.w + rhs.w, self.h + rhs.h,).into()
	}
}

impl Div<f64,> for Coord2d {
	type Output = Self;

	fn div(self, rhs: f64,) -> Self::Output {
		Self { w: self.w / rhs, h: self.h / rhs, }
	}
}

impl Div<f64,> for &Coord2d {
	type Output = Coord2d;

	fn div(self, rhs: f64,) -> Self::Output {
		(self.h / rhs, self.w / rhs,).into()
	}
}

impl From<(f64, f64,),> for Coord2d {
	fn from(value: (f64, f64,),) -> Self {
		Self { w: value.0, h: value.1, }
	}
}

struct Rgb {
	r: u8,
	g: u8,
	b: u8,
}

impl Rgb {
	pub fn new(r: u8, g: u8, b: u8,) -> Self {
		Self { r, g, b, }
	}
}

impl Add for Rgb {
	type Output = Self;

	fn add(self, rhs: Self,) -> Self::Output {
		Self { r: self.r + rhs.r, g: self.g + rhs.g, b: self.b + rhs.b, }
	}
}

impl Add for &Rgb {
	type Output = Rgb;

	fn add(self, rhs: Self,) -> Self::Output {
		(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b,).into()
	}
}

impl Add<(u8, u8, u8,),> for Rgb {
	type Output = Self;

	fn add(self, rhs: (u8, u8, u8,),) -> Self::Output {
		Self { r: self.r + rhs.0, g: self.g + rhs.1, b: self.b + rhs.2, }
	}
}

impl Add<(u8, u8, u8,),> for &Rgb {
	type Output = Rgb;

	fn add(self, rhs: (u8, u8, u8,),) -> Self::Output {
		(self.r + rhs.0, self.g + rhs.1, self.b + rhs.2,).into()
	}
}

impl From<(u8, u8, u8,),> for Rgb {
	fn from(value: (u8, u8, u8,),) -> Self {
		Self { r: value.0, g: value.1, b: value.2, }
	}
}

trait CanvasDrawBasic {
	fn triangle(
		&self,
		vert1: &Coord2d,
		vert2: &Coord2d,
		vert3: &Coord2d,
		stroke_color: Option<&Rgb,>,
		fill_color: Option<&Rgb,>,
	);

	fn triangle_filled(
		&self,
		vert1: &Coord2d,
		vert2: &Coord2d,
		vert3: &Coord2d,
		stroke_color: Option<&Rgb,>,
		fill_color: &Rgb,
	) {
		self.triangle(vert1, vert2, vert3, stroke_color, Some(fill_color,),);
	}

	fn triangle_empty(
		&self,
		vert1: &Coord2d,
		vert2: &Coord2d,
		vert3: &Coord2d,
		stroke_color: Option<&Rgb,>,
	) {
		self.triangle(vert1, vert2, vert3, stroke_color, None,);
	}
}

impl CanvasDrawBasic for CanvasRenderingContext2d {
	fn triangle(
		&self,
		vert1: &Coord2d,
		vert2: &Coord2d,
		vert3: &Coord2d,
		stroke_color: Option<&Rgb,>,
		fill_color: Option<&Rgb,>,
	) {
		self.move_to(vert1.w, vert1.h,);
		self.begin_path();
		self.line_to(vert2.w, vert2.h,);
		self.line_to(vert3.w, vert3.h,);
		self.line_to(vert1.w, vert1.h,);
		self.close_path();

		if let Some(Rgb { r, g, b, },) = stroke_color {
			self.set_stroke_style_str(&format!("rgb({r} {g} {b})"),);
		}
		self.stroke();

		if let Some(Rgb { r, g, b, },) = fill_color {
			self.set_fill_style_str(&format!("rgb({r} {g} {b})"),);
			self.fill();
		}
	}
}
