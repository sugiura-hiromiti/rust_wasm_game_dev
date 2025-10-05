use crate::engn::Image;
use crate::engn::Renderer;
use anyhow::Result as Rslt;
use wasm_bindgen::prelude::*;
use web_sys::console;

// mod archv;

// this attribute enables using `log!` macro when `brwsr` module is used
#[macro_use]
mod brwsr;
mod engn;

type JRslt<T,> = Result<T, JsValue,>;

trait ContainerFixer {
	type ErrFlipped;
	type Flipped;
	type Consumed;
	type Transposed;

	fn trans(self,) -> Self::Transposed;

	fn flip_err(self,) -> Self::ErrFlipped;
	fn flip(self,) -> Self::Flipped;

	fn consume_with<O,>(
		self,
		success_op: impl FnOnce(Self::Consumed,) -> O,
	) -> Option<O,>;

	fn consume(self,) -> Option<Self::Consumed,>
	where Self: std::marker::Sized {
		self.consume_with(|c| c,)
	}

	// NOTE: add flattern functionality
}

impl<T, E1: std::fmt::Debug, E2: std::fmt::Debug,> ContainerFixer
	for Result<Result<T, E1,>, E2,>
{
	type Consumed = T;
	type ErrFlipped = Result<Result<T, E2,>, E1,>;
	type Flipped = Self;
	type Transposed = Self::ErrFlipped;

	fn trans(self,) -> Self::Transposed {
		self.flip_err()
	}

	fn flip(self,) -> Self::Flipped {
		self
	}

	fn flip_err(self,) -> Self::ErrFlipped {
		match self {
			Ok(Ok(t,),) => Ok(Ok(t,),),
			Ok(Err(e1,),) => Err(e1,),
			Err(e2,) => Ok(Err(e2,),),
		}
	}

	fn consume_with<O,>(self, success_op: impl FnOnce(T,) -> O,) -> Option<O,> {
		match self {
			Ok(Ok(t,),) => Some(success_op(t,),),
			Ok(Err(e1,),) => {
				console::error_1(&JsValue::from_str(&format!("{e1:#?}"),),);
				None
			},
			Err(e2,) => {
				console::error_1(&JsValue::from_str(&format!("{e2:#?}"),),);
				None
			},
		}
	}
}

impl<T, E: std::fmt::Debug,> ContainerFixer for Option<Result<T, E,>,> {
	type Consumed = T;
	type ErrFlipped = Self;
	type Flipped = Result<Option<T,>, E,>;
	type Transposed = Self::Flipped;

	fn trans(self,) -> Self::Transposed {
		self.flip()
	}

	fn flip(self,) -> Self::Flipped {
		self.transpose()
	}

	fn flip_err(self,) -> Self::ErrFlipped {
		self
	}

	fn consume_with<O,>(
		self,
		success_op: impl FnOnce(Self::Consumed,) -> O,
	) -> Option<O,> {
		match self {
			Some(Ok(t,),) => Some(success_op(t,),),
			Some(Err(e,),) => {
				log!("{e:?}",);
				None
			},
			None => None,
		}
	}
}

fn draw() -> Rslt<(),> {
	let async_block = async move {
		let draw_inner = async || -> Rslt<(),> {
			let rndrr = Renderer::new("game_canvas",).await?;
			let img = Image::new_sprite_sheet().await?;

			let mut frame_count = -1;
			let intrvl_cb = Closure::wrap(Box::new(move || {
				frame_count = (frame_count + 1) % 8;
				let intrvl_cb_inner = || -> Rslt<(),> {
					rndrr.clear();

					let frame_name = format!("Run ({}).png", frame_count + 1);
					rndrr.draw_sprite_sheet(&img, &frame_name, 300.0, 300.0,)?;
					Ok((),)
				};

				match intrvl_cb_inner() {
					Ok(_,) => {},
					Err(e,) => log!("{e:?}"),
				};
			},) as Box<dyn FnMut(),>,);

			brwsr::window_obj()?
				.set_interval_with_callback_and_timeout_and_arguments_0(
					intrvl_cb.as_ref().unchecked_ref(),
					50,
				)
				.unwrap();
			intrvl_cb.forget();

			Ok((),)
		};

		if let Err(e,) = draw_inner().await {
			log!("error happen while drawing: {e}");
			// console::error_1(&e,);
		}
	};

	brwsr::spawn_local(async_block,);

	Ok((),)
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> JRslt<(),> {
	console_error_panic_hook::set_once();

	draw().unwrap();

	log!("wasm end");

	Ok((),)
}
