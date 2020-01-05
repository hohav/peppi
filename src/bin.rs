use std::env;

fn main() {
	let args: Vec<String> = env::args().collect();
	println!("{:?}", slippi::parse(&args[1]));
}
