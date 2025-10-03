use std::ops::Add;
use std::ops::Div;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;
use web_sys::Event;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlImageElement;
use web_sys::console;

type JRslt<T,> = Result<T, JsValue,>;

macro_rules! console_log {
    ($($args:expr),*) => {
        $(
			  console::log_1(&JsValue::from_str($args,),)
		  );*
    };
}

trait BrowserContext<T,> {
	fn brwsr_ctx(self, err_msg: &str,) -> JRslt<T,>;
	fn dom_ctx(self, err_msg: &str,) -> JRslt<T,>
	where Self: std::marker::Sized {
		self.brwsr_ctx(&("dom operation failed: ".to_string() + err_msg),)
	}

	fn ctx_ctx(self, err_msg: &str,) -> JRslt<T,>
	where Self: std::marker::Sized {
		self.brwsr_ctx(&("failed to get context: ".to_owned() + err_msg),)
	}
}

impl<T,> BrowserContext<T,> for Option<T,> {
	fn brwsr_ctx(self, err_msg: &str,) -> JRslt<T,> {
		match self {
			Self::Some(v,) => Ok(v,),
			Self::None => Err(JsValue::from_str(err_msg,),),
		}
	}
}

fn get_canvas_element() -> JRslt<HtmlCanvasElement,> {
	let canvas = web_sys::window()
		.dom_ctx("window object not found",)?
		.document()
		.dom_ctx("document object not found",)?
		.get_element_by_id("game_canvas",)
		.dom_ctx("element with  id `game_canvas` not exist",)?;
	let canvas = HtmlCanvasElement::unchecked_from_js(JsValue::from(canvas,),);

	Ok(canvas,)
}

trait CanvasMethodsTyped {
	fn context_of(&self, context_id: &str,)
	-> JRslt<CanvasRenderingContext2d,>;
}

impl CanvasMethodsTyped for HtmlCanvasElement {
	fn context_of(
		&self, context_id: &str,
	) -> JRslt<CanvasRenderingContext2d,> {
		let ctx = self
			.get_context(context_id,)?
			.ctx_ctx(&format!("context id {context_id} does not supported",),)?;
		let ctx =
			CanvasRenderingContext2d::unchecked_from_js(JsValue::from(ctx,),);
		Ok(ctx,)
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

fn draw() -> JRslt<(),> {
	let canvas = get_canvas_element()?;
	let ctx = canvas.context_of("2d",)?;

	let async_block = async move {
		let draw_inner = async || -> JRslt<(),> {
			let (success_tx, rx,) =
				futures::channel::oneshot::channel::<JRslt<(),>,>();
			let success_tx = Rc::new(Mutex::new(Some(success_tx,),),);
			let error_tx = success_tx.clone();

			let success_cb = Closure::once(move |event: &Event| {
				console::log_1(event,);
				if let Some(tx,) = success_tx
					.lock()
					.ok()
					.and_then(|mut acq_mutex| acq_mutex.take(),)
				{
					tx.send(Ok((),),).expect(
						"failed to send success message of loading asset",
					);
				}
			},);
			let error_cb = Closure::once(move |err| {
				if let Some(tx,) = error_tx
					.lock()
					.ok()
					.and_then(|mut acq_mutex| acq_mutex.take(),)
				{
					tx.send(Err(err,),).expect(
						"failed to send error message of loading asset",
					);
				}
			},);

			let image = HtmlImageElement::new()?;

			// set callback when loading asset finished
			image.set_onload(Some(success_cb.as_ref().unchecked_ref(),),);
			image.set_onerror(Some(error_cb.as_ref().unchecked_ref(),),);

			image.set_src("Idle (1).png",);
			match rx.await {
				Ok(sent_msg,) => match sent_msg {
					Ok(_,) => ctx
						.draw_image_with_html_image_element(&image, 0.0, 0.0,)?,
					Err(e,) => {
						console_log!("error happen while loading asset");
						console::error_1(&e,)
					},
				},
				Err(e,) => {
					let e = JsValue::from_str(&e.to_string(),);
					console_log!(
						"error happen while sending message from callback"
					);
					console::error_1(&e,);
				},
			};

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
			let fill_color =
				(fill_color[0], fill_color[1], fill_color[2],).into();
			draw_sierpinski(&ctx, &vert1, &vert2, &vert3, &fill_color, 6,);
			Ok((),)
		};

		if let Err(e,) = draw_inner().await {
			console_log!("error happen while drawing!!!!!!!!!");
			console::error_1(&e,);
		}
	};

	wasm_bindgen_futures::spawn_local(async_block,);

	Ok((),)
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> JRslt<(),> {
	console_error_panic_hook::set_once();

	console_log!("wth");

	draw()?;

	console_log!("wasm end", "goodbye");

	Ok((),)
}
