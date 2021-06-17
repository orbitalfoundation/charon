
use std::fs;
use crossbeam::channel::*;
use service::*;

use quick_js::{Context, JsValue, console::Level };

#[derive(Clone)]
pub struct Scripting {}
impl Scripting {
	pub fn new() -> Box<dyn Serviceable> {
		Box::new(Self{})
	}
}
impl Serviceable for Scripting {
	fn name(&self) -> &str { "Scripting" }
	fn stop(&self) {}
	fn start(&self, _name: String, _sid: SID, send: Sender<Message>, recv: Receiver<Message> ) {
		let send = send.clone();
		let recv = recv.clone();
		let name = self.name();
		let _thread = std::thread::Builder::new().name(name.to_string()).spawn(move || {

			// Start javascript engine
			let context = Context::builder()
				.memory_limit(100000)
				.console(|level: Level, args: Vec<JsValue>| { println!("{}: {:?}", level, args); })
				.build()
				.unwrap();

			// javascript sleep helper
			// TODO later can do something like this : https://www.programmersought.com/article/13424789131/
			let orbital_sleep = move |duration:i32| {
				println!("javascript asked for sleep of duration {}",duration);
	            std::thread::sleep(std::time::Duration::from_millis(duration as u64));
				12341234
			};
			context.add_callback("sleep", orbital_sleep ).unwrap();

			// javascript message pipeline helper
			let orbital_message = move |_a:String| {
				let message = Message::Event("/display".to_string(),_a.to_string());
				let send2 = send.clone();
				send2.send(message).expect("error");
				12341234
			};
			context.add_callback("orbital_message", orbital_message ).unwrap();

			// add some other special helpers to the context as well - these happen to be written in js
			let contents = fs::read_to_string("../public/index.js").expect("Something went wrong reading the file");
			let value = context.eval_as::<String>(&contents).unwrap();
			println!("result is {}",&value);

			// unused stuff - tbd

//            let contents = fs::read_to_string("scripts/bootsupport.js").expect("Something went wrong reading the file");

			// wait for commands and then load and run those arbitrary scripts
			// TODO right now this is not inside of a message handler - move there later
			// TODO remove hardcoded test
/*
			let mut contents = fs::read_to_string("../apps/test3d/weathercard.js").expect("Something went wrong reading the file").to_owned();
			let str2:String =  "orbital_script_parser(0,0,root);\n\"done\";\n".to_owned();
			contents.push_str(&str2);
			let value2 = context.eval_as::<String>(&contents).unwrap();
			println!("result is {}",&value2);
*/
			// um... ? not sure if I should do anything here... if js file returns is it done? what about on_event handling?

			while let Ok(message) = recv.recv() {
				match message {
					_ => { },
				}
			}
		});
	}
}

