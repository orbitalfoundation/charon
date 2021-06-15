
use broker;
use display;
use scripting;

fn main() {

	let services = [
		broker::Broker::new,
		scripting::Scripting::new,
		display::Display::new,		// must be last due to a defect in winit
	];

	broker::bootstrap( &services )
}


