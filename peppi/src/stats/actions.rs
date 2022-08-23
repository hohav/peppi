use std::collections::VecDeque;
use crate::stats::interface::StatComputer;
use crate::model::{
	game::{self, Start, Frames},
	slippi::Version,
	frame::{Frame, Post},
	enums::action_state::*,
};

#[derive(Clone, Default, Debug)]
pub struct ActionComputer {
	last_frame_processed: i32,
	player_stat_states: Vec<PlayerStatState>,
}

#[derive(Clone, Debug)]
struct PlayerStatState {
	actions: ActionStat,
	prev_age: f32,
	last_states: VecDeque<State>,
}

impl Default for PlayerStatState {
	fn default() -> Self {
		let actions = Default::default();
		let prev_age = -1.0;
		let mut last_states = VecDeque::new();
		last_states.resize_with(8, Default::default);

		PlayerStatState {
			actions,
			prev_age,
			last_states,
		}
	}
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct ActionStat {
	pub jab1: u16,
	pub jab2: u16,
	pub jab3: u16,
	pub jabm: u16,
	pub dash_attack: u16,
	pub ftilt: u16,
	pub utilt: u16,
	pub dtilt: u16,
	pub fsmash: u16,
	pub usmash: u16,
	pub dsmash: u16,
	pub nair: u16,
	pub fair: u16,
	pub bair: u16,
	pub uair: u16,
	pub dair: u16,
	pub fthrow: u16,
	pub bthrow: u16,
	pub uthrow: u16,
	pub dthrow: u16,
	pub roll: u16,
	pub spot_dodge: u16,
	pub air_dodge: u16,
	pub ledge_grab: u16,
	pub ground_tech_neutral: u16,
	pub ground_tech_roll: u16,
	pub ground_bounce: u16,
	pub wall_tech_neutral: u16,
	pub wall_tech_jump: u16,
	pub wall_bounce: u16,
	pub ceiling_tech: u16,
	pub ceiling_bounce: u16,
	pub dash_dance: u16,
	pub wavedash: u16,
	pub waveland: u16,
	pub grab: u16,
	pub grab_success: u16,
	pub l_cancel: Option<LCancel> // v2.0.0 and later only
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct LCancel {
	pub success: u16,
	pub fail: u16,
}

impl StatComputer for ActionComputer {
	type Stat = Vec<ActionStat>;

	// Requires action state frame counter field to work properly
	const MIN_VERSION: Version = Version(0, 2, 0);

	fn new(start: &Start) -> Self {
		if start.slippi.version < Self::MIN_VERSION {
			panic!("Minimum version required: {} given: {}", Self::MIN_VERSION, start.slippi.version);
		}
		let last_frame_processed = game::FIRST_FRAME_INDEX - 1;
		let mut player_stat_states: Vec<PlayerStatState> = Vec::new();
		player_stat_states.resize_with(start.players.len(), Default::default);

		ActionComputer {
			last_frame_processed,
			player_stat_states,
		}
	}

	fn process(&mut self, frames: &Frames) {
		match frames {
			Frames::P1(fs) => self.process_impl(fs),
			Frames::P2(fs) => self.process_impl(fs),
			Frames::P3(fs) => self.process_impl(fs),
			Frames::P4(fs) => self.process_impl(fs),
		}
	}

	fn into_inner(self) -> Self::Stat {
		self.player_stat_states.into_iter().map(|s| s.actions).collect()
	}
}

impl ActionComputer {
	fn process_impl<const N: usize>(&mut self, frames: &[Frame<N>]) {
		for frame in frames {
			if frame.index <= self.last_frame_processed {
				continue;
			}

			let stat_state_iter = self.player_stat_states.iter_mut();
			let post_iter = frame.ports.iter().map(|pd| pd.leader.post);
			for (stat_state, post) in stat_state_iter.zip(post_iter) {

				// get state/age values
				let prev_state = *stat_state.last_states.front().unwrap();
				let prev_age = stat_state.prev_age;
				let curr_state = post.state;
				let curr_age = post.state_age.unwrap();

				let is_new_action = curr_state != prev_state || prev_age > curr_age;

				// update stat_state for the next frame
				// we pop from back and push to front so .get(n) returns the 
				// nth most recent state (0 index is current state)
				stat_state.last_states.pop_back();
				stat_state.last_states.push_front(curr_state);
				stat_state.prev_age = curr_age;

				if !is_new_action {
					continue;
				}

				stat_state.count_actions(&post);
			}
		}
	}
}

impl PlayerStatState {
	fn count_actions(&mut self, post: &Post) {
		let actions: &mut ActionStat = &mut self.actions;
		match post.state {
			State::Common(s) => match s {
				// Basic attacks
				Common::ATTACK_11 => actions.jab1 += 1,
				Common::ATTACK_12 => actions.jab2 += 1,
				Common::ATTACK_13 => actions.jab3 += 1,
				Common::ATTACK_100_START => actions.jabm += 1,
				Common::ATTACK_DASH => actions.dash_attack += 1,
				s if is_ftilt(s) => actions.ftilt += 1,
				Common::ATTACK_HI_3 => actions.utilt += 1,
				Common::ATTACK_LW_3 => actions.dtilt += 1,
				s if is_fsmash(s) => actions.fsmash += 1,
				Common::ATTACK_HI_4 => actions.usmash += 1,
				Common::ATTACK_LW_4 => actions.dsmash += 1,
				Common::ATTACK_AIR_N => actions.nair += 1,
				Common::ATTACK_AIR_F => actions.fair += 1,
				Common::ATTACK_AIR_B => actions.bair += 1,
				Common::ATTACK_AIR_HI => actions.uair += 1,
				Common::ATTACK_AIR_LW => actions.dair += 1,

				// Throws
				Common::THROW_F => actions.fthrow += 1,
				Common::THROW_B => actions.bthrow += 1,
				Common::THROW_HI => actions.uthrow += 1,
				Common::THROW_LW => actions.dthrow += 1,

				// Dodges
				Common::ESCAPE_F |
				Common::ESCAPE_B => actions.roll += 1,
				Common::ESCAPE => actions.spot_dodge += 1,
				Common::ESCAPE_AIR => actions.air_dodge += 1,

				// Techs
				Common::PASSIVE => actions.ground_tech_neutral += 1,
				Common::PASSIVE_STAND_F |
				Common::PASSIVE_STAND_B => actions.ground_tech_roll += 1,
				Common::PASSIVE_CEIL => actions.ceiling_tech += 1,
				Common::DOWN_BOUND_U |
				Common::DOWN_BOUND_D => actions.ground_bounce += 1,
				Common::FLY_REFLECT_WALL => actions.wall_bounce += 1,
				Common::FLY_REFLECT_CEIL => actions.ceiling_bounce += 1,

				// Other
				Common::CLIFF_CATCH => actions.ledge_grab += 1,
				Common::DASH if is_dash_dance(&self.last_states) => actions.dash_dance += 1,

				_ => ()

			},

			State::GameAndWatch(s) => match s {
				// GnW has standard moves coded as special moves
				GameAndWatch::JAB => actions.jab1 += 1,
				GameAndWatch::RAPID_JABS_START => actions.jabm += 1,
				GameAndWatch::DOWN_TILT => actions.dtilt += 1,
				GameAndWatch::SIDE_SMASH => actions.fsmash += 1,
				GameAndWatch::NAIR => actions.nair += 1,
				GameAndWatch::BAIR => actions.bair += 1,
				GameAndWatch::UAIR => actions.uair += 1,
				_ => ()
			},

			State::Peach(s) => match s {
				// Peach fsmashes are coded as special moves
				Peach::SIDE_SMASH_GOLF_CLUB |
				Peach::SIDE_SMASH_FRYING_PAN |
				Peach::SIDE_SMASH_TENNIS_RACKET => actions.fsmash += 1,
				_ => ()
			},
			_ => (),
		}

		self.handle_wavething();
		self.handle_grab();
		self.handle_l_cancel(post);
		self.handle_wall_tech();
	}

	// Share code for wavedash and waveland
	fn handle_wavething(&mut self) {
		// Must be in special landing
		let curr_state: State = *self.last_states.get(0).unwrap();
		if curr_state != State::Common(Common::LANDING_FALL_SPECIAL) {
			return;
		}

		// Previous state must have been airdodge or a jump/falling state
		let prev_state: State = *self.last_states.get(1).unwrap();
		if let State::Common(prev_state) = prev_state {
			if !(prev_state == Common::ESCAPE_AIR ||
				(prev_state >= Common::KNEE_BEND &&
				prev_state <= Common::FALL_AERIAL_B)) {
				return;
			}
		}

		// If many of the previous states are an airdodge then it was probably
		// an airdodge (it's a long animation)
		if self.last_states.range(1..8).all(|s| *s == State::Common(Common::ESCAPE_AIR)) {
			return;
		}

		// Don't count imperfect wavelands/wavedashes as an air dodge action
		if self.last_states.range(1..8).any(|s| *s == State::Common(Common::ESCAPE_AIR)) {
			self.actions.air_dodge -= 1;
		}

		// Recent knee bend means it's a wavedash
		// Otherwise, it's a waveland
		if self.last_states.range(1..8).any(|s| *s == State::Common(Common::KNEE_BEND)) {
			self.actions.wavedash += 1;
		} else {
			self.actions.waveland += 1;
		}
	}

	fn handle_grab(&mut self) {
		let curr_state: State = *self.last_states.get(0).unwrap();
		let prev_state: State = *self.last_states.get(1).unwrap();
		match curr_state {
			State::Common(Common::CATCH) => self.actions.grab += 1,
			State::Common(Common::CATCH_DASH) => {
				self.actions.grab += 1;
				// Don't count boost grabs as dash attack
				if prev_state == State::Common(Common::ATTACK_DASH) {
					self.actions.dash_attack -= 1;
				}
			},
			State::Common(s) if is_grab_action(s) => {
				if let State::Common(prev_state) = prev_state {
					if is_grabbing(prev_state) {
						self.actions.grab_success += 1;
					}
				}
			},
			_ => ()
		}
	}

	fn handle_l_cancel(&mut self, post: &Post) {
		match (&mut self.actions.l_cancel, post.l_cancel, post.state) {
			(None, Some(_), _) => {
				self.actions.l_cancel = Some(LCancel { success: 0, fail: 0 });
			},
			(Some(l_cancel_stat), Some(Some(l_cancel_post)), State::Common(state))
			if is_aerial_landing(state) => {
				match l_cancel_post {
					true => l_cancel_stat.success += 1,
					false => l_cancel_stat.fail += 1,
				}
			}
			_ => (),
		}
	}

	fn handle_wall_tech(&mut self) {
		let curr_state: State = *self.last_states.get(0).unwrap();
		let prev_state: State = *self.last_states.get(1).unwrap();

		match curr_state {
			State::Common(Common::PASSIVE_WALL) => self.actions.wall_tech_neutral += 1,
			State::Common(Common::PASSIVE_WALL_JUMP) => {
				self.actions.wall_tech_jump += 1;
				// Don't double count if we were in passive before
				if prev_state == State::Common(Common::PASSIVE_WALL) {
					self.actions.wall_tech_neutral -= 1;
				}
			},
			_ => ()
		}
	}
}

fn is_ftilt(state: Common) -> bool {
	state >= Common::ATTACK_S_3_HI && state <= Common::ATTACK_S_3_LW
}

fn is_fsmash(state: Common) -> bool {
	state >= Common::ATTACK_S_4_HI && state <= Common::ATTACK_S_4_LW
}

fn is_aerial_landing(state: Common) -> bool {
	state >= Common::LANDING_AIR_N || state <= Common::LANDING_AIR_LW
}

fn is_grabbing(state: Common) -> bool {
	state == Common::CATCH || state == Common::CATCH_DASH
}

// returns true for any state that could happen after a successful grab
fn is_grab_action(state: Common) -> bool {
	state > Common::CATCH && state <= Common::THROW_LW && state != Common::CATCH_DASH
}

fn is_dash_dance(last_states: &VecDeque<State>) -> bool {
	*last_states.get(0).unwrap() == State::Common(Common::DASH) &&
	*last_states.get(1).unwrap() == State::Common(Common::TURN) &&
	*last_states.get(2).unwrap() == State::Common(Common::DASH)
}
