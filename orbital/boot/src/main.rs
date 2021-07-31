
use broker;
use scripting;
use view;

fn main() {

	let services = [
		broker::Broker::new,
		scripting::Scripting::new,
		view::View::new,		// must be last due to a design issue in winit
	];

	broker::bootstrap( &services )
}


