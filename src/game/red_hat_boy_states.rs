use crate::engn::Point;
use crate::game::RedHatBoyStateMachine;
use std::marker::PhantomData;

const FLOOR: i16 = 475;

#[derive(Clone, Copy,)]
pub(super) struct RedHatBoyState<S,> {
	context: RedHatBoyContext,
	#[doc(hidden)]
	_state:  PhantomData<S,>,
}

impl<S,> RedHatBoyState<S,> {
	pub(super) fn update(&mut self,) {
		self.context.update();
	}
}

#[derive(Clone, Copy,)]
pub(super) struct RedHatBoyContext {
	pub frame: u8,
	pub pos:   Point,
	pub vel:   Point,
}

impl RedHatBoyContext {
	fn update(&mut self,) {
		self.frame = self.frame.wrapping_add(1,);
		self.pos += self.vel;
	}
}

#[derive(Clone, Copy,)]
pub(super) struct Idle;
#[derive(Clone, Copy,)]
pub(super) struct Running;

impl RedHatBoyState<Idle,> {
	pub fn new() -> Self {
		Self {
			context: RedHatBoyContext {
				frame: 0,
				pos:   Point { x: 0, y: FLOOR, },
				vel:   Point { x: 0, y: 0, },
			},
			_state:  PhantomData,
		}
	}

	pub fn run(self, vel: Point,) -> RedHatBoyState<Running,> {
		let mut context = self.context;
		context.frame = 0;
		context.vel = vel;
		RedHatBoyState { context, _state: PhantomData, }
	}

	pub fn reset(&mut self,) {
		self.context.vel = Point { x: 0, y: 0, };
	}
}

impl RedHatBoyState<Running,> {
	pub fn with_velocity(mut self, vel: Point,) -> Self {
		self.context.vel = vel;
		self
	}

	pub fn idle(self,) -> RedHatBoyState<Idle,> {
		let mut context = self.context;
		context.frame = 0;
		context.vel = Point { x: 0, y: 0, };
		RedHatBoyState { context, _state: PhantomData, }
	}
}

impl RedHatBoyStateMachine {
	pub(super) fn context(&self,) -> &RedHatBoyContext {
		match self {
			Self::Idle(red_hat_boy_state,) => &red_hat_boy_state.context,
			Self::Running(red_hat_boy_state,) => &red_hat_boy_state.context,
		}
	}
}
