use std::env;

fn main() {
	let args: Vec<String> = env::args().collect();
	let game = slippi::parse(&args[1]).unwrap();
	println!("{:#?}", game);

	if args.len() > 2 {
		let port = args[2].parse::<usize>().unwrap();
		let frame = args[3].parse::<usize>().unwrap();
		if let Some(port) = &game.ports[port] {
			println!("{:#?}", &port.leader.pre[frame]);
			println!("{:#?}", &port.leader.post[frame]);
		}
	}
}
