mod app;
mod cmd;
mod matcher;

use app::Cmd;

fn main() {
	if let Err(e) = Cmd::from_args().run() {
		eprintln!("error: {}", e);
		std::process::exit(2);
	}
}
