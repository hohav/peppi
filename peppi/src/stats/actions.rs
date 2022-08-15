use std::collections::VecDeque;
use crate::stats::interface::StatComputer;
use crate::model::{
    game::{self, Start, Frames},
    frame::Frame,
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

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct ActionStat {
	pub jab1: u16,
	pub jab2: u16,
	pub jab3: u16,
	pub jabm: u16,
	pub dash: u16,
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
    pub spotdodge: u16,
    pub airdodge: u16,
    pub ledgegrab: u16,
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
                stat_state.last_three_states.push_back(curr_state);
                stat_state.last_state_age = curr_age;

                if !is_new_action {
                    continue;
                }

                stat_state.count_actions(curr_state, &post);
            }
        }
    }
}

impl PlayerStatState {
    fn count_actions(&mut self, curr_state: State, _post: &Post) -> () {
        let actions: &mut ActionStat = &mut self.actions;
        match curr_state {
            State::Common(Common::ATTACK_11) => actions.jab1 += 1,
            State::Common(Common::ATTACK_12) => actions.jab2 += 1,
            State::Common(Common::ATTACK_13) => actions.jab3 += 1,
            State::Common(Common::ATTACK_100_START) => actions.jabm += 1,
            State::Common(Common::ATTACK_DASH) => actions.dash += 1,
            State::Common(s) if s.0 >= Common::ATTACK_S_3_HI.0 && s.0 <= Common::ATTACK_S_3_LW.0 => actions.ftilt += 1,
            State::Common(Common::ATTACK_HI_3) => actions.utilt += 1,
            State::Common(Common::ATTACK_LW_3) => actions.dtilt += 1,
            State::Common(s) if s.0 >= Common::ATTACK_S_4_HI.0 && s.0 <= Common::ATTACK_S_4_LW.0 => actions.ftilt += 1,
            State::Common(Common::ATTACK_HI_4) => actions.usmash += 1,
            State::Common(Common::ATTACK_LW_4) => actions.dsmash += 1,
            State::Common(Common::ATTACK_AIR_N) => actions.nair += 1,
            State::Common(Common::ATTACK_AIR_F) => actions.fair += 1,
            State::Common(Common::ATTACK_AIR_B) => actions.bair += 1,
            State::Common(Common::ATTACK_AIR_HI) => actions.uair += 1,
            State::Common(Common::ATTACK_AIR_LW) => actions.dair += 1,

            // GnW has standard moves coded as special moves
            State::GameAndWatch(GameAndWatch::JAB) => actions.jab1 += 1,
            State::GameAndWatch(GameAndWatch::RAPID_JABS_START) => actions.jabm += 1,
            State::GameAndWatch(GameAndWatch::DOWN_TILT) => actions.dtilt += 1,
            State::GameAndWatch(GameAndWatch::SIDE_SMASH) => actions.fsmash += 1,
            State::GameAndWatch(GameAndWatch::NAIR) => actions.nair += 1,
            State::GameAndWatch(GameAndWatch::BAIR) => actions.bair += 1,
            State::GameAndWatch(GameAndWatch::UAIR) => actions.uair += 1,

            // Peach fsmashes are coded as special moves
            State::Peach(Peach::SIDE_SMASH_GOLF_CLUB) => actions.fsmash += 1,
            State::Peach(Peach::SIDE_SMASH_FRYING_PAN) => actions.fsmash += 1,
            State::Peach(Peach::SIDE_SMASH_TENNIS_RACKET) => actions.fsmash += 1,

            // Throws
            State::Common(Common::THROW_F) => actions.fthrow += 1,
            State::Common(Common::THROW_B) => actions.bthrow += 1,
            State::Common(Common::THROW_HI) => actions.uthrow += 1,
            State::Common(Common::THROW_LW) => actions.dthrow += 1,

            // Other
            State::Common(Common::ESCAPE_F) => actions.roll += 1,
            State::Common(Common::ESCAPE_B) => actions.roll += 1,
            State::Common(Common::ESCAPE) => actions.spotdodge += 1,
            //State::Common(Common::ESCAPE_AIR) => actions.airdodge += 1,
            State::Common(Common::CLIFF_CATCH) => actions.ledgegrab += 1,

            _ => (),
        }
    }
}
