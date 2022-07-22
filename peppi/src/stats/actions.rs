use std::collections::VecDeque;
use crate::stats::interface::StatComputer;
use crate::model::{
    game::{self, Start, Frames},
    frame::Frame,
    enums::action_state::{State, Common},
};

#[derive(Clone, Default, Debug)]
pub struct ActionComputer {
    last_frame_processed: i32,
    player_stat_states: Vec<PlayerStatState>,
}

#[derive(Clone, Debug)]
struct PlayerStatState {
    actions: ActionStat,
    last_state_age: f32,
	last_three_states: VecDeque<State>,
}

impl Default for PlayerStatState {
    fn default() -> Self {
        let actions = Default::default();
        let last_state_age = -1.0;
        let mut last_three_states = VecDeque::new();
        last_three_states.resize_with(3, Default::default);

        PlayerStatState {
            actions,
            last_state_age,
            last_three_states,
        }
    }
}

// TODO add more fields/subfields
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct ActionStat {
	jab1: u16,
	jab2: u16,
	jab3: u16,
	jabm: u16,
	dash: u16,
	ftilt: u16,
	utilt: u16,
	dtilt: u16,
	fsmash: u16,
	usmash: u16,
	dsmash: u16,
	nair: u16,
	fair: u16,
	bair: u16,
	uair: u16,
	dair: u16,
}

impl StatComputer for ActionComputer {
	type Stat = Vec<ActionStat>;

	fn new(start: &Start) -> Self {
        let last_frame_processed = game::FIRST_FRAME_INDEX - 1;
		let mut player_stat_states: Vec<PlayerStatState> = Vec::new();
        player_stat_states.resize_with(start.players.len(), Default::default);

        ActionComputer {
            last_frame_processed,
            player_stat_states,
        }
	}

	fn process(&mut self, frames: &Frames) -> () {
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
    fn process_impl<const N: usize>(&mut self, frames: &[Frame<N>]) -> () {
        for frame in frames {
            if frame.index <= self.last_frame_processed {
                continue;
            }

            let stat_state_iter = self.player_stat_states.iter_mut();
            let post_iter = frame.ports.iter().map(|pd| pd.leader.post);
            for (stat_state, post) in stat_state_iter.zip(post_iter) {

                // get state/age values
                let last_state = stat_state.last_three_states.back().unwrap();
                let last_age = stat_state.last_state_age;
                let curr_state = post.state;
                let curr_age = post.state_age.unwrap(); // TODO handle old versions

                let is_new_action = curr_state != *last_state || last_age > curr_age;
                drop(last_state);

                // update state_state for the next frame
                stat_state.last_three_states.pop_front();
                stat_state.last_three_states.push_back(post.state);
                stat_state.last_state_age = curr_age;

                if !is_new_action {
                    continue;
                }

                // TODO streamline this pattern
                match curr_state {
                    State::Common(Common::ATTACK_11) => stat_state.actions.jab1 += 1,
                    State::Common(Common::ATTACK_12) => stat_state.actions.jab2 += 1,
                    State::Common(Common::ATTACK_13) => stat_state.actions.jab3 += 1,
                    State::Common(Common::ATTACK_100_START) => stat_state.actions.jabm += 1,
                    State::Common(Common::ATTACK_DASH) => stat_state.actions.dash += 1,
                    State::Common(Common::ATTACK_S_3_HI) => stat_state.actions.ftilt += 1,
                    State::Common(Common::ATTACK_S_3_HI_S) => stat_state.actions.ftilt += 1,
                    State::Common(Common::ATTACK_S_3_S) => stat_state.actions.ftilt += 1,
                    State::Common(Common::ATTACK_S_3_LW_S) => stat_state.actions.ftilt += 1,
                    State::Common(Common::ATTACK_S_3_LW) => stat_state.actions.ftilt += 1,
                    State::Common(Common::ATTACK_HI_3) => stat_state.actions.utilt += 1,
                    State::Common(Common::ATTACK_LW_3) => stat_state.actions.dtilt += 1,
                    State::Common(Common::ATTACK_S_4_HI) => stat_state.actions.fsmash += 1,
                    State::Common(Common::ATTACK_S_4_HI_S) => stat_state.actions.fsmash += 1,
                    State::Common(Common::ATTACK_S_4_S) => stat_state.actions.fsmash += 1,
                    State::Common(Common::ATTACK_S_4_LW_S) => stat_state.actions.fsmash += 1,
                    State::Common(Common::ATTACK_S_4_LW) => stat_state.actions.fsmash += 1,
                    State::Common(Common::ATTACK_HI_4) => stat_state.actions.usmash += 1,
                    State::Common(Common::ATTACK_LW_4) => stat_state.actions.dsmash += 1,
                    State::Common(Common::ATTACK_AIR_N) => stat_state.actions.nair += 1,
                    State::Common(Common::ATTACK_AIR_F) => stat_state.actions.fair += 1,
                    State::Common(Common::ATTACK_AIR_B) => stat_state.actions.bair += 1,
                    State::Common(Common::ATTACK_AIR_HI) => stat_state.actions.uair += 1,
                    State::Common(Common::ATTACK_AIR_LW) => stat_state.actions.dair += 1,
                    //TODO GnW & Peach quirks
                    _ => (),
                }

            }
        }
    }
}
