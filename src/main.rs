mod runner;

fn main() {
	// aici vom pune bufferul cu machine code
	let v = Vec::<u8>::new();
	runner::run(&v);
}
