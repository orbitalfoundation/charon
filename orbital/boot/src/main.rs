
use broker;
use scripting;
use graphics;

fn main() {

	let services = [
		broker::Broker::new,
		scripting::Scripting::new,
		graphics::Graphics::new,		// must be last due to a design issue in winit
	];

	broker::bootstrap( &services )
}


