
use broker;
use scripting;
//use display;
use graphics;

fn main() {

	let services = [
		broker::Broker::new,
		scripting::Scripting::new,
		graphics::Graphics::new,		// must be last due to a defect in winit
	];

	broker::bootstrap( &services )
}


