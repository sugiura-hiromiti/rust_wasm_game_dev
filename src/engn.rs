use crate::ContainerFixer;
use crate::brwsr;
use crate::brwsr::Canvas;
use crate::brwsr::Fetch;
use crate::brwsr::Rslt;
use crate::brwsr::get_canvas_element;
use crate::brwsr::new_image;
use crate::brwsr::raf_closure;
use crate::brwsr::request_animation_frame;
use anyhow::Context;
use anyhow::anyhow;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::unbounded;
use futures::channel::oneshot::Canceled;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Add;
use std::ops::AddAssign;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::HtmlImageElement;
use web_sys::KeyboardEvent;

/// path to sprite sheet
const SPRITE_SHEET: &str = "rhb.png";
/// path to sprite sheet mapper
const SPRITE_SHEET_MAPPER: &str = "rhb.json";
const FRAME_SIZE: f32 = 1.0 / 60.0 * 1000.0;

pub struct Renderer {
	drawer: brwsr::Renderer,
}

impl Renderer {
	pub async fn new(id: &str,) -> Rslt<Self,> {
		let drawer = get_canvas_element(id,)?.renderer("2d",)?;
		Ok(Self { drawer, },)
	}

	pub fn draw_image(&self, img: &Image,) -> Rslt<(),> {
		// let sprite = self
		// 	.sprite_sheet_mapper
		// 	.get(name,)
		// 	.ok_or_else(|| anyhow!("{name} not found in sprite sheet"),)?;

		self.drawer.draw_image(&img.image,)
	}

	pub fn draw_sprite_sheet(
		&self,
		img: &Image,
		name: &str,
		pos: Point,
	) -> Rslt<(),> {
		let frame = &img
			.sprite_sheet_mapper
			.as_ref()
			.context("Image struct is not sprite sheet",)?
			.get(name,)
			.ok_or_else(|| anyhow!("{name} not found in sprite sheet"),)?
			.frame;

		self.drawer.draw_image_opt(
			&img.image,
			frame.x_f64(),
			frame.y_f64(),
			pos.x_into(),
			pos.y_into(),
			frame.w_f64(),
			frame.h_f64(),
		)
	}

	pub fn clear(&self,) {
		self.drawer.clear();
	}
}

pub struct Image {
	image:               HtmlImageElement,
	sprite_sheet_mapper: Option<Sheet,>,
}

impl Image {
	pub async fn new() -> Rslt<Self,> {
		let image = new_image()?;
		// load(&image, src,).await??;
		Ok(Self { image, sprite_sheet_mapper: None, },)
	}

	pub async fn new_sprite_sheet() -> Rslt<Self,> {
		let image = new_image()?;
		load(&image, SPRITE_SHEET,).await??;

		let sprite_sheet_mapper = Some(sprite_sheet_mapper().await?,);
		Ok(Self { image, sprite_sheet_mapper, },)
	}

	pub async fn set_sprite_sheet(&mut self,) -> Rslt<&Self,> {
		self.sprite_sheet_mapper = Some(sprite_sheet_mapper().await?,);
		load(&self.image, SPRITE_SHEET,).await??;
		Ok(self,)
	}

	pub async fn load(&self, src: &str,) -> Rslt<&Self,> {
		load(&self.image, src,).await??;
		Ok(self,)
	}
}

#[derive(Deserialize, Debug,)]
pub struct Sheet {
	frames: HashMap<String, Sprite,>,
}

impl Sheet {
	pub fn get<'a,>(&'a self, key: &str,) -> Option<&'a Sprite,> {
		self.frames.get(key,)
	}
}

#[derive(Deserialize, Debug,)]
pub struct Sprite {
	frame: Rect,
}

#[derive(Deserialize, Debug,)]
pub struct Rect {
	x: u16,
	y: u16,
	w: u16,
	h: u16,
}

impl Rect {
	pub fn x_f64(&self,) -> f64 {
		self.x.into()
	}

	pub fn y_f64(&self,) -> f64 {
		self.y.into()
	}

	pub fn w_f64(&self,) -> f64 {
		self.w.into()
	}

	pub fn h_f64(&self,) -> f64 {
		self.h.into()
	}
}

pub async fn sprite_sheet_mapper() -> Rslt<Sheet,> {
	brwsr::window_obj()?.fetch_json_de(SPRITE_SHEET_MAPPER,).await
}

async fn set_sprite_sheet(
	to: &HtmlImageElement,
) -> Rslt<Result<(), Canceled,>,> {
	load(to, SPRITE_SHEET,).await
}

async fn load(
	to: &HtmlImageElement,
	src: &str,
) -> Rslt<Result<(), Canceled,>,> {
	let (success_tx, rx,) = futures::channel::oneshot::channel::<Rslt<(),>,>();
	let success_tx = Rc::new(Mutex::new(Some(success_tx,),),);
	let error_tx = success_tx.clone();

	let success_cb = brwsr::closure_once(move |_event: &Event| {
		if let Some(tx,) =
			success_tx.lock().ok().and_then(|mut acq_mutex| acq_mutex.take(),)
		{
			tx.send(Ok((),),)
				.expect("failed to send success message of loading asset",);
		}
	},);
	let error_cb = brwsr::closure_once(move |err: &Event| {
		if let Some(tx,) =
			error_tx.lock().ok().and_then(|mut acq_mutex| acq_mutex.take(),)
		{
			tx.send(Err(anyhow!("{err:?}"),),)
				.expect("failed to send error message of loading asset",);
		}
	},);

	// set callback when loading asset finished
	to.set_onload(Some(success_cb.as_ref().unchecked_ref(),),);
	to.set_onerror(Some(error_cb.as_ref().unchecked_ref(),),);

	to.set_src(src,);

	// flipping container here enables using `?` shorthand on functions
	// web_sys provides
	rx.await.flip_err()
}

pub trait Game: std::marker::Sized {
	async fn init(&mut self,) -> Rslt<(),>;
	fn update(&mut self, kb_state: &KeyboardState,);
	fn draw(&self,);
}

pub struct GameLoop {
	last_frame:        f64,
	accumulated_delta: f32,
}

impl GameLoop {
	pub async fn start(mut game: impl Game + 'static,) -> Rslt<(),> {
		let mut kbe_rx = prepare_input()?;
		game.init().await?;
		let mut game_loop =
			Self { last_frame: brwsr::now()?, accumulated_delta: 0.0, };

		let f = Rc::new(RefCell::new(None,),);
		let g = f.clone();

		let mut kb_stat = KeyboardState::new();
		*f.borrow_mut() = Some(raf_closure(move |perf| {
			kb_stat.process_input(&mut kbe_rx,);
			game_loop.accumulated_delta += (perf - game_loop.last_frame) as f32;

			while game_loop.accumulated_delta > FRAME_SIZE {
				game.update(&kb_stat,);
				game_loop.accumulated_delta -= FRAME_SIZE;
			}
			game_loop.last_frame = perf;
			game.draw();
			request_animation_frame(g.borrow().as_ref().unwrap(),)
				.unwrap_or_else(|e| {
					panic!(
						"on frame: {perf}, error happen while requesting \
						 animation frame: {e}"
					)
				},);
		},),);

		request_animation_frame(
			f.borrow().as_ref().context("game loop is none",)?,
		)?;

		Ok((),)
	}
}

#[derive(Debug,)]
enum KeyState {
	Up(KeyboardEvent,),
	Down(KeyboardEvent,),
}

fn prepare_input() -> Rslt<UnboundedReceiver<KeyState,>,> {
	let (tx, rx,) = unbounded();
	let keydown_tx = Rc::new(RefCell::new(tx,),);
	let keyup_tx = keydown_tx.clone();

	let onkeydown = brwsr::closure_new::<_, dyn FnMut(KeyboardEvent,),>(
		move |kbe: KeyboardEvent| {
			keydown_tx
				.borrow_mut()
				.start_send(KeyState::Down(kbe,),)
				.expect("failed to send keydown state",);
		},
	);
	let onkeyup = brwsr::closure_new::<_, dyn FnMut(KeyboardEvent,),>(
		move |kbe: KeyboardEvent| {
			keyup_tx
				.borrow_mut()
				.start_send(KeyState::Up(kbe,),)
				.expect("failed to send keyup state",);
		},
	);

	let canvas = get_canvas_element("game_canvas",)?;
	canvas.set_onkeyup(Some(onkeyup.as_ref().unchecked_ref(),),);
	onkeyup.forget();
	canvas.set_onkeydown(Some(onkeydown.as_ref().unchecked_ref(),),);
	onkeydown.forget();

	Ok(rx,)
}

pub struct KeyboardState {
	pressed_keys: HashMap<String, KeyboardEvent,>,
}

impl KeyboardState {
	fn new() -> Self {
		Self { pressed_keys: HashMap::new(), }
	}

	pub fn is_pressed(&self, code: &str,) -> bool {
		self.pressed_keys.contains_key(code,)
	}

	fn set_pressed(&mut self, code: &str, event: KeyboardEvent,) {
		self.pressed_keys.insert(code.into(), event,);
	}

	fn set_released(&mut self, code: &str,) {
		self.pressed_keys.remove(code,);
	}

	fn process_input(&mut self, kbe_rx: &mut UnboundedReceiver<KeyState,>,) {
		loop {
			match kbe_rx.try_next() {
				Ok(Some(KeyState::Down(event,),),) => {
					self.set_pressed(&event.code(), event,)
				},
				Ok(Some(KeyState::Up(event,),),) => {
					self.set_released(&event.code(),)
				},
				_ => {
					// log!("prsd kys: {:?}", self.pressed_keys);
					break;
				},
			}
		}
	}
}

#[derive(Clone, Copy,)]
pub struct Point {
	pub x: i16,
	pub y: i16,
}

impl Point {
	pub fn x_into(&self,) -> f64 {
		self.x.into()
	}

	pub fn y_into(&self,) -> f64 {
		self.y.into()
	}
}

impl Add for Point {
	type Output = Self;

	fn add(mut self, rhs: Self,) -> Self::Output {
		self += rhs;
		self
	}
}

impl AddAssign for Point {
	fn add_assign(&mut self, rhs: Self,) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}
