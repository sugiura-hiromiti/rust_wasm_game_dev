use crate::engn::Point;
use crate::game::RedHatBoyStateMachine;
use std::marker::PhantomData;

const FLOOR: i16 = 475;

const RUN_SPEED: i16 = 3;
/// velocity of idle state
const IDLE_VEL: Point = Point { x: 0, y: 0, };
/// velocity of run state
const RUN_VEL: Point = Point { x: RUN_SPEED, y: 0, };
/// velocity of slide state
const SLIDE_VEL: Point = RUN_VEL;

#[derive(Clone, Copy,)]
pub(super) struct RedHatBoyState<S,> {
	context: RedHatBoyContext,
	#[doc(hidden)]
	_state:  PhantomData<S,>,
}

impl<S,> RedHatBoyState<S,> {}

#[derive(Clone, Copy,)]
pub(super) struct RedHatBoyContext {
	pub frame: u8,
	pub pos:   Point,
}

impl RedHatBoyContext {
	fn update(&mut self, vel: Point,) {
		self.frame = self.frame.wrapping_add(1,);
		self.pos += vel;
	}

	fn reset_frame(mut self,) -> Self {
		self.frame = 0;
		self
	}
}

#[derive(Clone, Copy,)]
pub(super) struct Idle;
#[derive(Clone, Copy,)]
pub(super) struct Running;
#[derive(Clone, Copy,)]
pub(super) struct Sliding;

impl RedHatBoyState<Idle,> {
	pub fn new() -> Self {
		Self {
			context: RedHatBoyContext {
				frame: 0,
				pos:   Point { x: 0, y: FLOOR, },
			},
			_state:  PhantomData,
		}
	}

	pub fn run(self,) -> RedHatBoyState<Running,> {
		RedHatBoyState {
			context: self.context.reset_frame(),
			_state:  PhantomData,
		}
	}

	pub fn update(&mut self,) {
		self.context.update(IDLE_VEL,);
	}
}

impl RedHatBoyState<Running,> {
	pub fn idle(self,) -> RedHatBoyState<Idle,> {
		RedHatBoyState {
			context: self.context.reset_frame(),
			_state:  PhantomData,
		}
	}

	pub fn slide(self,) -> RedHatBoyState<Sliding,> {
		RedHatBoyState {
			context: self.context.reset_frame(),
			_state:  PhantomData,
		}
	}

	pub fn update(&mut self,) {
		self.context.update(RUN_VEL,);
	}
}

impl RedHatBoyState<Sliding,> {
	pub fn idle(self,) -> RedHatBoyState<Idle,> {
		RedHatBoyState {
			context: self.context.reset_frame(),
			_state:  PhantomData,
		}
	}

	pub fn run(self,) -> RedHatBoyState<Running,> {
		RedHatBoyState {
			context: self.context.reset_frame(),
			_state:  PhantomData,
		}
	}

	pub fn update(&mut self,) {
		self.context.update(SLIDE_VEL,);
	}
}

impl RedHatBoyStateMachine {
	pub(super) fn context(&self,) -> &RedHatBoyContext {
		match self {
			Self::Idle(red_hat_boy_state,) => &red_hat_boy_state.context,
			Self::Running(red_hat_boy_state,) => &red_hat_boy_state.context,
			Self::Sliding(red_hat_boy_state,) => &red_hat_boy_state.context,
		}
	}
}
