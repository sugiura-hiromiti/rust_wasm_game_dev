use crate::Rslt;
use crate::engn::Game;
use crate::engn::Image;
use crate::engn::KeyboardState;
use crate::engn::Renderer;
use crate::game::red_hat_boy_states::Idle;
use crate::game::red_hat_boy_states::RedHatBoyState;
use crate::game::red_hat_boy_states::Running;
use crate::game::red_hat_boy_states::Sliding;

mod red_hat_boy_states;

/// count of idle cards
const IDLE_CARDS: u8 = 10;
/// count of running cards
const RUN_CARDS: u8 = 8;
/// count of sliding cards
const SLIDE_CARDS: u8 = 5;

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

		let pressed = |code| kb_state.is_pressed(code,);

		let event = if pressed("KeyD",) {
			GameEvent::Slide
		} else if pressed("KeyF",) {
			GameEvent::Run
		} else {
			GameEvent::Idle
		};

		rhb.update(event,);
	}

	fn draw(&self,) {
		let Some(ref rndrr,) = self.renderer else {
			log!("no renderer");
			return;
		};
		rndrr.clear();
		if let Some(rhb,) = self.rhb.as_ref() {
			rhb.draw(rndrr,).expect("error happen while drawing rhb",);
		}
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

	fn update(&mut self, event: GameEvent,) {
		self.state_machine = self.state_machine.transition(event,);
		self.state_machine = self.state_machine.update();
	}
}

#[derive(Clone, Copy, strum::Display,)]
enum RedHatBoyStateMachine {
	Idle(RedHatBoyState<Idle,>,),
	#[strum(to_string = "Run")]
	Running(RedHatBoyState<Running,>,),
	#[strum(to_string = "Slide")]
	Sliding(RedHatBoyState<Sliding,>,),
}

impl From<RedHatBoyState<Running,>,> for RedHatBoyStateMachine {
	fn from(value: RedHatBoyState<Running,>,) -> Self {
		Self::Running(value,)
	}
}

impl From<RedHatBoyState<Idle,>,> for RedHatBoyStateMachine {
	fn from(value: RedHatBoyState<Idle,>,) -> Self {
		Self::Idle(value,)
	}
}

impl From<RedHatBoyState<Sliding,>,> for RedHatBoyStateMachine {
	fn from(value: RedHatBoyState<Sliding,>,) -> Self {
		Self::Sliding(value,)
	}
}

#[derive(Clone, Copy, Debug, Default,)]
pub enum GameEvent {
	#[default]
	Idle,
	Run,
	Slide,
}

impl RedHatBoyStateMachine {
	fn transition(self, event: GameEvent,) -> Self {
		match (self, event,) {
			(Self::Idle(state,), GameEvent::Run,) => state.run().into(),
			(Self::Running(state,), GameEvent::Idle,) => state.idle().into(),
			(Self::Running(state,), GameEvent::Slide,) => state.slide().into(),
			(Self::Sliding(state,), GameEvent::Idle,) => state.idle().into(),
			(Self::Sliding(state,), GameEvent::Run,) => state.run().into(),
			_ => self,
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
			Self::Sliding(_,) => SLIDE_CARDS,
		}
	}

	fn update(self,) -> Self {
		match self {
			Self::Idle(mut state,) => {
				state.update();
				Self::Idle(state,)
			},
			Self::Running(mut state,) => {
				state.update();
				Self::Running(state,)
			},
			Self::Sliding(mut state,) => {
				state.update();
				Self::Sliding(state,)
			},
		}
	}
}
