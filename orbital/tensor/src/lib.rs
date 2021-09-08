
use crossbeam::channel::*;
use service::*;

extern crate rustface;
use rustface::{Detector, ImageData};
const BUFSIZE : usize = 1280*720/4;



#[derive(Clone)]
pub struct Tensor {}
impl Tensor {
	pub fn new() -> Box<dyn Serviceable> {
		Box::new(Self{})
	}
}
impl Serviceable for Tensor {
    fn name(&self) -> &str { "Tensor" }
	fn stop(&self) {}
	fn start(&self, _name: String, _sid: SID, send: Sender<Message>, recv: Receiver<Message> ) {
		let send = send.clone();
		let recv = recv.clone();
		let name = self.name();
		let _thread = std::thread::Builder::new().name(name.to_string()).spawn(move || {


			// start a detector

	        let mut detector = rustface::create_detector("../public/resources/seeta_fd_frontal_v1.0.bin").unwrap();
	        detector.set_min_face_size(20);
	        detector.set_score_thresh(2.0);
	        detector.set_pyramid_scale_factor(0.8);
	        detector.set_slide_window_step(4, 4);
	        println!("loaded face detector");

	        let mut buffer = Box::new([0u8;BUFSIZE]);


	
			// in this sketch the pretend tensor module listens to ALL camera frames and looks for faces as a built in capability (like recognizing qr codes)
			// TODO arguably like the camera service this should only work on a given frame and only pipe back to the a specified caller
			//let message = Message::Subscribe(_sid,"/frames".to_string());
		    //send.send(message).expect("error");

		    // listen to view
			//send.send(Message::Subscribe(_sid,"/view".to_string())).expect("tensor: failed to subscribe");

// TODO
// if it waits for every frame then it will get pretty far behind
// throw away frames more elegantly

			let mut count:i32 = 0;

	        while let Ok(message) = recv.recv() {
			    match message {
			    	Message::Event(topic,data) => {
			    		println!("Face: Got raw data: {} {}",topic, data);
			    		let message = Message::Event("/view".to_string(),"[Face->Display: here is a face]".to_string());
						send.send(message).expect("error");
			    	},

	                Message::Share(sharedmemory) => {

	                	// TODO gah
	                	count = count + 1;
	                	if(count > 100 ) {

	                		count = 0;

		                	// get memory
		                    let mut ptr = sharedmemory.lock().unwrap();

		                    // copy it
						    for y in 0..360{
						        for x in 0..640{
						            let pixel = ptr[y*1280*2+x*2];
						            //let pixel = pixel.swap_bytes(); //.rotate_right(8);  // target format is ARGB ignoring A, and src format is probaby RGBA
						            let pixel = pixel as u8;
						            buffer[y*640+x]=pixel;
						        }
						    }
						    let mut image = ImageData::new(buffer.as_mut(), 640, 360);

						    // detect face

						    for face in detector.detect(&mut image).into_iter() {
						        let x = 2 * face.bbox().x() as usize;
						        let y = 2 * face.bbox().y() as usize;
						        let w = 2 * face.bbox().width() as usize;
						        let h = 2 * face.bbox().height() as usize;
							    println!("done face detection");
						    }
						}


	                },


			        _ => { },
			    }
	        }
		});
	}
}



