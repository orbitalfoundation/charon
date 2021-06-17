
use std::fs;
use crossbeam::channel::*;
use service::*;

use bevy::prelude::*;

#[derive(Clone)]
pub struct Graphics {}
impl Graphics {
	pub fn new() -> Box<dyn Serviceable> {
		Box::new(Self{})
	}
}
impl Serviceable for Graphics {
	fn name(&self) -> &str { "Graphics" }
	fn stop(&self) {}
	fn start(&self, _name: String, _sid: SID, send: Sender<Message>, recv: Receiver<Message> ) {
		let send = send.clone();
		let recv = recv.clone();
		let name = self.name();

		// run bevy
		App::build()
	        .insert_resource(WindowDescriptor {
	            title: "I am a window!".to_string(),
	            width: 500.,
	            height: 300.,
	            vsync: true,
	            ..Default::default()
	        })
			.insert_resource(Msaa { samples: 4 })
			.add_plugins(DefaultPlugins)
	        .add_system(change_title.system())
			.add_startup_system(setup_bevy.system())
			.run();

/*
		// this never gets visited
		let _thread = std::thread::Builder::new().name(name.to_string()).spawn(move || {

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

*/

	}
}



/// This system will then change the title during execution
fn change_title(time: Res<Time>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_title(format!(
        "Seconds since startup: {}",
        time.seconds_since_startup().round()
    ));
}


fn setup_bevy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
