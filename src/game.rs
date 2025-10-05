use crate::Rslt;
use crate::engn::Game;
use crate::engn::Image;
use crate::engn::KeyboardState;
use crate::engn::Point;
use crate::engn::Renderer;

pub struct WalkTheDog {
	renderer: Option<Renderer,>,
	image:    Option<Image,>,
	pos:      Point,
	frame:    u8,
}

impl WalkTheDog {
	pub fn new() -> Self {
		Self {
			renderer: None,
			image:    None,
			pos:      Point { x: 0, y: 0, },
			frame:    0,
		}
	}
}

impl Game for WalkTheDog {
	async fn init(&mut self,) -> Rslt<(),> {
		self.renderer = Some(Renderer::new("game_canvas",).await?,);
		self.image = Some(Image::new_sprite_sheet().await?,);
		Ok((),)
	}

	fn update(&mut self, kb_stat: &KeyboardState,) {
		if self.frame < 23 {
			self.frame += 1;
		} else {
			self.frame = 0;
		}

		let mut vel = Point { x: 0, y: 0, };
		let pressed = |code| kb_stat.is_pressed(code,);
		if pressed("KeyA",) {
			vel.x -= 3;
		}
		if pressed("KeyS",) {
			vel.y += 3;
		}
		if pressed("KeyD",) {
			vel.y -= 3;
		}
		if pressed("KeyF",) {
			vel.x += 3;
		}

		self.pos += vel;
	}

	fn draw(&self,) {
		let frame_name = format!("Run ({}).png", (self.frame / 3) + 1);

		if let Some(r,) = self.renderer.as_ref() {
			r.clear();
			if let Some(img,) = self.image.as_ref() {
				r.draw_sprite_sheet(img, &frame_name, self.pos,)
					.expect("failed to draw sheet",);
			}
		}
	}
}
