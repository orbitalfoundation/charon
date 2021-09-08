
use crossbeam::channel::*;
use service::*;

use pixels::{Pixels, SurfaceTexture}; // see https://nyxtom.dev/2020/10/07/winit-rust/

use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;


const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;

//
// A renderable
//
struct Renderable {
	kind: i16,
	box_x: i16,
	box_y: i16,
	velocity_x: i16,
	velocity_y: i16,
}


#[derive(Clone)]
pub struct ViewPixels {}
impl ViewPixels {
	pub fn new() -> Box<dyn Serviceable> {
		Box::new(Self{})
	}
}
impl Serviceable for ViewPixels {
	fn name(&self) -> &str { "View" }
	fn stop(&self) {}
	fn start(&self, _name: String, _sid: SID, send: Sender<Message>, recv: Receiver<Message> ) {

		let _send = send.clone();
		let _recv = recv.clone();
		let _name = self.name();

	    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
		// This is an array of things to render

		let mut objects: Vec<Renderable> = vec![];

		// clear backgrounder
		objects.push( Renderable {
        	kind: 0,
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        });

	    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
		// Build 

		let mut input = WinitInputHelper::new();

		let event_loop = EventLoop::new();

		let window = {
			let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
			WindowBuilder::new()
				.with_title("View")
				.with_inner_size(size)
				.with_min_inner_size(size)
				.build(&event_loop)
				.unwrap()
		};

		let window_size = window.inner_size();
		let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
		let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();

	    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
		// tell the system message broker that I want to listen for messages to '/view'
		// TODO - i wonder if the broker cannot be smarter? think about wiring a bit more
		// TODO - also, I could just tell the message system what my receive port is at this time; not earlier
		// TOOD - or... send to this based on absolute path /system/context/view or something?

		let message = Message::Subscribe(_sid,"/view".to_string());
	    send.send(message).expect("error");

	    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
		// Run forever

		event_loop.run(move |event, _, control_flow| {


			// TODO
			//		- catch messages here and then add new objects to some kind of list
			//		- i could test with 2d boxes
			//		- how hard is it to tell wgpu to make and cache say a box, or text, or a 3d object?
			//		- how hard is it to dynamically extend or add new things to the list of wgpu objects?
			//		- it is a big hassle to do camera projection, shader lighting transforms and so on?
			//		- is it a big hassle to load fonts
			//		- is it a big hassle to load gltf?
			//		- see https://nyxtom.dev/2020/10/08/framebuffers/

	        while let Ok(message) = recv.try_recv() {
	            match message {
	                Message::Event(topic,data) => {
	                    println!("ViewPixels: Received: {} {}",topic, data);
	                    match data.as_str() {
	                        "cube" => {
			                    println!("ViewPixels: got a cube");
								let r = Renderable {
						        	kind: 1,
						            box_x: 24,
						            box_y: 16,
						            velocity_x: 1,
						            velocity_y: 1,
						        };
								objects.push(r);
	                        },
	                        _ => {
	                        }
	                    }
	                },
	                //Message::Share(sharedmemory) => {
	                //},
					_ => { },
				}
			}

			// Draw the current frame
			if let Event::RedrawRequested(_) = event {

				for i in 0..objects.len() {

					let r = &mut objects[i];

					r.draw(pixels.get_frame());

					r.update();
				}

				if pixels.render().is_err() {
					*control_flow = ControlFlow::Exit;
					return;
				}

			}

			// Handle input events
			if input.update(&event) {
				// Close events
				if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
					*control_flow = ControlFlow::Exit;
					return;
				}

				// Resize the window
				if let Some(size) = input.window_resized() {
					pixels.resize_surface(size.width, size.height);
				}

				// Redraw
				window.request_redraw();
			}
		});
	}
}



impl Renderable {

    /// Update the internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            if self.kind == 1 {

	            let inside_the_box = x >= self.box_x
	                && x < self.box_x + BOX_SIZE
	                && y >= self.box_y
	                && y < self.box_y + BOX_SIZE;

	            let rgba = if inside_the_box {
	                [0x5e, 0x48, 0xe8, 0xff]
	            } else {
	            	continue
	            };

	            pixel.copy_from_slice(&rgba);
	        }
            if self.kind == 0 {

	            let rgba = [0x48, 0xb2, 0xe8, 0xff];

	            pixel.copy_from_slice(&rgba);
	        }

        }
    }
}
