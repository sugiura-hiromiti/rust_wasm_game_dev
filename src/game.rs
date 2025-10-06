use crate::Rslt;
use crate::engn::Game;
use crate::engn::Image;
use crate::engn::KeyboardState;
use crate::engn::Point;
use crate::engn::Renderer;
use crate::game::red_hat_boy_states::Idle;
use crate::game::red_hat_boy_states::RedHatBoyState;
use crate::game::red_hat_boy_states::Running;

mod red_hat_boy_states;

/// count of idle cards
const IDLE_CARDS: u8 = 10;
/// count of running cards
const RUN_CARDS: u8 = 8;
const WALK_SPEED: i16 = 3;

pub struct WalkTheDog {
	renderer: Option<Renderer,>,
	rhb:      Option<RedHatBoy,>,
}

impl WalkTheDog {
	pub fn new() -> Self {
		Self { renderer: None, rhb: None, }
	}
}

impl Game for WalkTheDog {
	async fn init(&mut self,) -> Rslt<(),> {
		self.renderer = Some(Renderer::new("game_canvas",).await?,);

		let image = Image::new_sprite_sheet().await?;

		self.rhb = Some(RedHatBoy::new(image,)?,);
		Ok((),)
	}

	fn update(&mut self, kb_state: &KeyboardState,) {
		let Some(ref mut rhb,) = self.rhb else {
			return;
		};

		rhb.update(kb_state,);
	}

	fn draw(&self,) {
		let Some(ref rndrr,) = self.renderer else {
			return;
		};
		rndrr.clear();
		if let Some(rhb,) = self.rhb.as_ref() {
			rhb.draw(rndrr,).expect("error happen while drawing rhb",);
		}
		// let frame_name = format!("Run ({}).png", (self.frame / 3) + 1);
		//
		// if let Some(r,) = self.renderer.as_ref() {
		// 	r.clear();
		// 	if let Some(RedHatBoy { image, .. },) = self.rhb.as_ref() {
		// 		r.draw_sprite_sheet(image, &frame_name, self.pos,)
		// 			.expect("failed to draw sheet",);
		// 	}
		// }
	}
}

struct RedHatBoy {
	state_machine: RedHatBoyStateMachine,
	image:         Image,
}

impl RedHatBoy {
	fn new(image: Image,) -> Rslt<Self,> {
		Ok(Self {
			state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new(),),
			image,
		},)
	}

	fn draw(&self, rndrr: &Renderer,) -> Rslt<(),> {
		rndrr.draw_sprite_sheet(
			&self.image,
			&self.state_machine.frame_name(),
			self.state_machine.context().pos,
		)
	}

	fn update(&mut self, kb_state: &KeyboardState,) {
		let vel = Self::keyboard_velocity(kb_state,);
		let event = if vel.x == 0 && vel.y == 0 {
			GameEvent::Idle
		} else {
			GameEvent::Run { vel, }
		};

		self.state_machine = self.state_machine.transition(event,);
		self.state_machine = self.state_machine.update();
	}

	fn keyboard_velocity(kb_state: &KeyboardState,) -> Point {
		let mut vel = Point { x: 0, y: 0, };
		let pressed = |code| kb_state.is_pressed(code,);
		if pressed("KeyA",) {
			vel.x -= WALK_SPEED;
		}
		if pressed("KeyF",) {
			vel.x += WALK_SPEED;
		}
		if pressed("KeyS",) {
			vel.y += WALK_SPEED;
		}
		if pressed("KeyD",) {
			vel.y -= WALK_SPEED;
		}

		vel
	}
}

#[derive(Clone, Copy, strum::Display,)]
enum RedHatBoyStateMachine {
	Idle(RedHatBoyState<Idle,>,),
	#[strum(to_string = "Run")]
	Running(RedHatBoyState<Running,>,),
}

impl From<RedHatBoyState<Running,>,> for RedHatBoyStateMachine {
	fn from(value: RedHatBoyState<Running,>,) -> Self {
		Self::Running(value,)
	}
}

#[derive(Clone, Copy,)]
pub enum GameEvent {
	Idle,
	Run { vel: Point, },
}

impl RedHatBoyStateMachine {
	fn transition(self, event: GameEvent,) -> Self {
		match (self, event,) {
			(Self::Idle(mut state,), GameEvent::Idle,) => {
				state.reset();
				Self::Idle(state,)
			},
			(Self::Idle(state,), GameEvent::Run { vel, },) => {
				state.run(vel,).into()
			},
			(Self::Running(state,), GameEvent::Idle,) => {
				Self::Idle(state.idle(),)
			},
			(Self::Running(state,), GameEvent::Run { vel, },) => {
				Self::Running(state.with_velocity(vel,),)
			},
		}
	}

	fn frame_name(&self,) -> String {
		let cur_card_count = (self.context().frame / 3) % self.card_count() + 1;
		format!("{} ({cur_card_count}).png", self)
	}

	fn card_count(&self,) -> u8 {
		match self {
			Self::Idle(_,) => IDLE_CARDS,
			Self::Running(_,) => RUN_CARDS,
		}
	}

	fn update(self,) -> Self {
		match self {
			Self::Idle(mut red_hat_boy_state,) => {
				red_hat_boy_state.update();
				Self::Idle(red_hat_boy_state,)
			},
			Self::Running(mut red_hat_boy_state,) => {
				red_hat_boy_state.update();
				Self::Running(red_hat_boy_state,)
			},
		}
	}
}
