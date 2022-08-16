use std::{fs, io};
use peppi::{
	model::game::Game,
	stats::{
		interface::StatComputer,
		actions::{ActionComputer, ActionStat},
	},
};

fn read_game(path: &str) -> Result<Game, String> {
	let mut buf = io::BufReader::new(
		fs::File::open(path).unwrap());
	peppi::game(&mut buf, None, None).map_err(|e| format!("couldn't deserialize game: {:?}", e))
}

fn game(name: &str) -> Result<Game, String> {
	read_game(&format!("tests/data/{}.slp", name))
}

fn actions(game: &Game) -> Vec<ActionStat> {
	let mut comp: ActionComputer = ActionComputer::new(&game.start);
	comp.process(&game.frames);
	comp.into_inner()
}

#[test]
fn throw_grab() -> Result<(), String> {
	let game = game("throw_grab")?;
	let actions: ActionStat = actions(&game).swap_remove(1);

	assert_eq!(actions.fthrow, 1);
	assert_eq!(actions.bthrow, 2);
	assert_eq!(actions.uthrow, 1);
	assert_eq!(actions.dthrow, 1);
	assert_eq!(actions.grab, 10);
	assert_eq!(actions.grab_success, 7);

	Ok(())
}

#[test]
fn gnw_actions() -> Result<(), String> {
	let game = game("gnw_actions")?;
	let actions: ActionStat = actions(&game).swap_remove(0);

	assert_eq!(actions.jab1, 2);
	assert_eq!(actions.jabm, 1);
	assert_eq!(actions.ftilt, 1);
	assert_eq!(actions.utilt, 1);
	assert_eq!(actions.dtilt, 1);
	assert_eq!(actions.fsmash, 1);
	assert_eq!(actions.usmash, 1);
	assert_eq!(actions.dsmash, 1);
	assert_eq!(actions.nair, 1);
	assert_eq!(actions.fair, 1);
	assert_eq!(actions.bair, 1);
	assert_eq!(actions.uair, 1);
	assert_eq!(actions.dair, 1);
	assert_eq!(actions.l_cancel.as_ref().unwrap().success, 2);
	assert_eq!(actions.l_cancel.as_ref().unwrap().fail, 0);

	Ok(())
}

#[test]
fn peach_fsmash() -> Result<(), String> {
	let game = game("peach_fsmash")?;
	let actions: ActionStat = actions(&game).swap_remove(0);

	assert_eq!(actions.fsmash, 4);
	Ok(())
}


#[test]
fn action_edge_cases() -> Result<(), String> {
	let game = game("action_edge_cases")?;
	let actions: ActionStat = actions(&game).swap_remove(0);

	assert_eq!(actions.jab1, 4);
	assert_eq!(actions.jab2, 3);
	assert_eq!(actions.jab3, 2);
	assert_eq!(actions.jabm, 1);
	assert_eq!(actions.grab, 5);
	assert_eq!(actions.grab_success, 4);
	assert_eq!(actions.dash_attack, 2);
	assert_eq!(actions.bair, 8);
	assert_eq!(actions.spot_dodge, 4);
	assert_eq!(actions.ftilt, 3);
	assert_eq!(actions.fsmash, 3);

	Ok(())
}

