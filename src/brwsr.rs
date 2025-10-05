use crate::JRslt;
pub use anyhow::Result as Rslt;
use anyhow::anyhow;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::IntoWasmClosure;
use wasm_bindgen::closure::WasmClosure;
use wasm_bindgen::closure::WasmClosureFnOnce;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::Document;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlImageElement;
use web_sys::Response;
use web_sys::Window;

macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into())
    };
}

pub trait ToAnyhow<T,> {
	fn to_anhw(self,) -> Rslt<T,>;
}

impl<T,> ToAnyhow<T,> for JRslt<T,> {
	fn to_anhw(self,) -> Rslt<T,> {
		self.map_err(|e| anyhow!("{e:?}"),)
	}
}

impl<T,> ToAnyhow<T,> for Result<T, serde_wasm_bindgen::Error,> {
	fn to_anhw(self,) -> Rslt<T,> {
		self.map_err(|e| anyhow!("{e}"),)
	}
}

pub trait BrowserContext<T,> {
	fn ctx(self, err_msg: &str,) -> JRslt<T,>;
	fn dom_ctx(self, err_msg: &str,) -> JRslt<T,>
	where Self: std::marker::Sized {
		self.ctx(&("dom operation failed: ".to_string() + err_msg),)
	}

	fn ctx_ctx(self, err_msg: &str,) -> JRslt<T,>
	where Self: std::marker::Sized {
		self.ctx(&("failed to get context: ".to_owned() + err_msg),)
	}

	fn brwsr_ctx(self, err_msg: &str,) -> JRslt<T,>
	where Self: std::marker::Sized {
		self.ctx(&("browser oriented error: ".to_owned() + err_msg),)
	}
}

impl<T,> BrowserContext<T,> for Option<T,> {
	fn ctx(self, err_msg: &str,) -> JRslt<T,> {
		self.ok_or(JsValue::from_str(err_msg,),)
	}
}

pub struct Renderer {
	ctx: web_sys::CanvasRenderingContext2d,
}

impl Renderer {
	const CANVAS_HEIGHT: f64 = 600.0;
	const CANVAS_WIDTH: f64 = 600.0;

	fn from_obj(val: JsValue,) -> Self {
		Self {
			ctx: web_sys::CanvasRenderingContext2d::unchecked_from_js(val,),
		}
	}

	pub fn clear(&self,) {
		self.ctx.clear_rect(0.0, 0.0, Self::CANVAS_WIDTH, Self::CANVAS_HEIGHT,);
	}

	#[allow(clippy::too_many_arguments)]
	pub fn draw_image_opt(
		&self,
		image: &HtmlImageElement,
		clip_x: f64,
		clip_y: f64,
		pos_x: f64,
		pos_y: f64,
		w: f64,
		h: f64,
	) -> Rslt<(),> {
		self.ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(image, clip_x, clip_y, w, h, pos_x, pos_y, w, h).to_anhw()
	}

	pub fn draw_image(&self, image: &HtmlImageElement,) -> Rslt<(),> {
		self.ctx.draw_image_with_html_image_element(image, 0.0, 0.0,).to_anhw()
	}
}

pub trait Canvas {
	fn renderer(&self, context_id: &str,) -> Rslt<Renderer,>;
}

impl Canvas for HtmlCanvasElement {
	fn renderer(&self, context_id: &str,) -> Rslt<Renderer,> {
		let ctx = self
			.get_context(context_id,)
			.to_anhw()?
			.ctx_ctx(&format!("context id {context_id} does not supported",),)
			.to_anhw()?;
		let ctx = Renderer::from_obj(JsValue::from(ctx,),);
		Ok(ctx,)
	}
}

pub fn window_obj() -> Rslt<Window,> {
	web_sys::window().dom_ctx("window object not found",).to_anhw()
}

pub fn document_obj_of(win: &Window,) -> Rslt<Document,> {
	win.document().dom_ctx("document object not found",).to_anhw()
}

pub fn document_obj() -> Rslt<Document,> {
	let win = window_obj()?;
	document_obj_of(&win,)
}

pub fn get_canvas_element(id: &str,) -> Rslt<HtmlCanvasElement,> {
	get_element_by_id(id,)
}

pub fn get_element_by_id<El: JsCast,>(id: &str,) -> Rslt<El,> {
	let elem = document_obj()?
		.get_element_by_id(id,)
		.ok_or(anyhow!("element with {id} not found"),)?;
	Ok(elem.unchecked_into(),)
}

pub fn new_image() -> Rslt<HtmlImageElement,> {
	HtmlImageElement::new().to_anhw()
}

pub fn closure_once<F, A, R, T,>(fn_once: F,) -> Closure<T,>
where
	F: WasmClosureFnOnce<T, A, R,>, // + 'static,
	T: WasmClosure + ?Sized,        // + 'static
{
	Closure::once(fn_once,)
}

pub fn closure_new<F, T,>(f: F,) -> Closure<T,>
where
	F: IntoWasmClosure<T,> + 'static,
	T: WasmClosure + ?Sized,
{
	Closure::new(f,)
}

/// raf stand for `request_animation_frame`
pub fn raf_closure(
	fn_mut: impl FnMut(f64,) + 'static,
) -> Closure<dyn FnMut(f64,),> {
	closure_new(fn_mut,)
}

pub fn request_animation_frame(cb: &Closure<dyn FnMut(f64,),>,) -> Rslt<i32,> {
	window_obj()?
		.request_animation_frame(cb.as_ref().unchecked_ref(),)
		.to_anhw()
}

pub fn spawn_local<F,>(future: F,)
where F: Future<Output = (),> + 'static {
	wasm_bindgen_futures::spawn_local(future,);
}

pub fn now() -> Rslt<f64,> {
	Ok(window_obj()?
		.performance()
		.dom_ctx("failed to get performance object",)
		.to_anhw()?
		.now(),)
}

pub trait Fetch {
	async fn raw_fetch(&self, src: &str,) -> Rslt<JsValue,>;

	async fn fetch_rsp(&self, src: &str,) -> Rslt<Response,> {
		let rsp = self.raw_fetch(src,).await?;
		Ok(Response::unchecked_from_js(rsp,),)
	}

	async fn fetch_json(&self, src: &str,) -> Rslt<JsValue,> {
		let rsp = self.fetch_rsp(src,).await?;
		JsFuture::from(
			rsp.json().map_err(|e| {
				anyhow!("could not get JSON from response: {e:?}")
			},)?,
		)
		.await
		.to_anhw()
	}

	async fn fetch_json_de<D: serde::de::DeserializeOwned,>(
		&self,
		src: &str,
	) -> Rslt<D,> {
		let json = self.fetch_json(src,).await?;
		let de = serde_wasm_bindgen::from_value(json,).to_anhw()?;
		Ok(de,)
	}
}

impl Fetch for Window {
	async fn raw_fetch(&self, src: &str,) -> Rslt<JsValue,> {
		JsFuture::from(self.fetch_with_str(src,),).await.to_anhw()
	}
}
