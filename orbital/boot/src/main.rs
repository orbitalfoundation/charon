

use crossbeam::channel::*;

use service::*;
use broker::*;
use camera::*;
use tensor::*;
use viewmakepad::*;


fn main() {

	//
	// in this pattern there's a slightly special service called a broker which has an inbound and outbound message channel
	// it is started as a thread (like every other service), and its message channels are passed to it; crossing the thread barrier
	//
	// the main point of all this is to get the broker thread in a position where it can listen to other requests
	// the whole point is to build up the critical "registry" of other services ... we have to have some kind of named list of services.
	// a different way to do this could be:
	// - i could try have the broker registery in the main thread (as a named hash)
	// - but the brokers message handling thread would have to somehow read across that thread barrier ... which is a PITA in rust
	//   basically, the subthread wants the registry; so there's no point in having the registry itself in the main thread
	//

    let (brokersend,brokerrecv) = unbounded::<Message>();
    let broker = Broker::new();
	broker.start("broker".to_string(),0,brokersend.clone(),brokerrecv.clone());

	// after the above - each hard coded service that i want to run gets passed its own message channels, AND a broker channel
	// so for example here I make a camera, with message handlers, and it will spin up its own thread to do work
	// this is is a kind of verbose version of what would be automated later - basically "load up hard coded device drivers"

	{
	    let sid: SID = rand::random::<SID>();
	    let (localsend,localrecv) = unbounded::<Message>();
	    let instance = Camera::new();
	    let name = instance.name().to_string();
	    let _ = brokersend.send(Message::Channel(sid,name.clone(),localsend));
	    instance.start(name,sid,brokersend.clone(),localrecv);
	}

    // tensor 
    // later this would be a wasm blob loaded off disk... 

    {
	    let sid: SID = rand::random::<SID>();
	    let (localsend,localrecv) = unbounded::<Message>();
	    let instance = Tensor::new();
	    let name = instance.name().to_string();
	    let _ = brokersend.send(Message::Channel(sid,name.clone(),localsend));
	    instance.start(name,sid,brokersend.clone(),localrecv);
	}

	// app
	// here i load up a script that describes an app (as a demo)
	// it looks like just another unit of computation
	// it could ask the broker to wire together pieces in a special way (building an app)
	// it can also have a javascript engine
	// TODO

    // due to an annoying issue with the way threads work, the graphics window thread has to be last

    {
	    let sid: SID = rand::random::<SID>();
	    let (localsend,localrecv) = unbounded::<Message>();
	    let instance = ViewMakepad::new();
	    let name = instance.name().to_string();
	    let _ = brokersend.send(Message::Channel(sid,name.clone(),localsend));
	    instance.start(name,sid,brokersend.clone(),localrecv);
	}

}



/*

sept 2021

	previously:
	+ i started a broker thread
	+ and then i told the broker about the message handlers for the item
	+ and then i started the item with the broker back channel

*

	what i would prefer:
	+ i just want a registry on main thread... which is highly problematic since the graphics thread grabs main thread...
	+ i pass the registry to each object and let it register itself... [ i guess i can do this with messages instead ]
	+ my app wiring code can just wire stuff directly [ i guess messages can also do this ]


*/







/*

// this is more like the pattern i would like to have.... sept 2021

//////////////////////////////
// A camera
//////////////////////////////

struct Camera {
	id: i32,
	name: String,
	nodes: Vec<Box<dyn Node>>,
}

impl Camera {
	pub fn new() -> Camera {
		let id: i32 = 1;
		let name = "camera".to_string();
		let nodes: Vec<Box<dyn Node>> = Vec::new();
		Self { id:id, name:name, nodes:nodes }
	}
}

impl Node for Camera {
	fn event(&self) {
	}
	fn nodes(&self) {
	}
	fn register(&self, node: &dyn Node) {
		// - given a parent - i guess i tell that parent about me
		// - 
	}
}

//////////////////////////////
// A tensor
//////////////////////////////

struct Tensor {
	id: i32,
	name: String,
	nodes: Vec<Box<dyn Node>>,
}

impl Tensor {
	pub fn new() -> Tensor {
		let id: i32 = 1;
		let name = "tensor".to_string();
		let nodes: Vec<Box<dyn Node>> = Vec::new();
		Self { id:id, name:name, nodes:nodes }
	}
	pub fn info(&self) -> i32 {
		self.id
	}
}

impl Node for Tensor {
	fn event(&self) {
	}
	fn nodes(&self) {
	}
	fn register(&self, node: &dyn Node) {
	}
}

//////////////////////////////
// A view
//////////////////////////////

struct Display {
	id: i32,
	name: String,
	nodes: Vec<Box<dyn Node>>,
}

impl Display {
	pub fn new() -> Display {
		let id: i32 = 1;
		let name = "display".to_string();
		let nodes: Vec<Box<dyn Node>> = Vec::new();
		Self { id:id, name:name, nodes:nodes }
	}
	pub fn info(&self) -> i32 {
		self.id
	}
}

impl Node for Display {
	fn event(&self) {
	}
	fn nodes(&self) {
	}
	fn register(&self, node: &dyn Node) {
	}
}

//////////////////////////////
// A sample app
//////////////////////////////

struct MyTestApp {
	id: i32,
	name: String,
	nodes: Vec<Box<dyn Node>>,
}

impl MyTestApp {
	pub fn new() -> MyTestApp {
		let id: i32 = 1;
		let name = "MyTestApp".to_string();
		let nodes: Vec<Box<dyn Node>> = Vec::new();
		Self { id:id, name:name, nodes:nodes }
	}
	pub fn info(&self) -> i32 {
		self.id
	}
}

impl Node for MyTestApp {
	fn event(&self) {
	}
	fn nodes(&self) {
	}
	fn register(&self, node: &dyn Node) {

		// in this very rough sketch i am implying a few things
		// - there are some named resources that can be produced or found; possibly with shorthand names, possibly with fully qualified urls+versions
		// - they expose inputs and outputs and can be wired together
		// - there would be some kind of permissions
		// - wiring is a capability of any node; it can route things to other things

//		node.wire("camera","tensor")
//		node.wire("tensor","display")
	}
}
*/
////////////////////////








