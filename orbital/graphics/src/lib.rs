
//////////////////////////////////////////////////////////////////////////////////////

use std::fs;
use crossbeam::channel::*;
use service::*;

use bevy::prelude::*;
use bevy_mod_picking::*;

mod orbit;
use orbit::{OrbitCamera,OrbitCameraPlugin};

//////////////////////////////////////////////////////////////////////////////////////

struct AWayToHaveGlobalState {
	receiver: Receiver<Message>,
}

struct MyProperties {
	x: f32,
	y: f32,
}

//////////////////////////////////////////////////////////////////////////////////////

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

	    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
		// tell the system message broker that I want to listen for messages to '/display'

		let message = Message::Subscribe(_sid,"/display".to_string());
	    send.send(message).expect("error");

	    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
		// run bevy - this never returns annoyingly

		App::new()
			.insert_resource(Msaa { samples: 4 })
			.insert_resource(WindowDescriptor {
			    title: "Orbital".to_string(),
			    width: 600.0,
			    height: 800.0,
			    ..Default::default()
			})
			.insert_resource(
				AWayToHaveGlobalState {
					receiver: recv.clone(),
				}
			)
			.add_system(set_title.system())
			//.add_startup_system(make_some_stuff.system())
			.add_system(listen_to_messages.system())
			.add_system(move_things.system())
			.add_plugins(DefaultPlugins)
			.add_plugin(OrbitCameraPlugin)
			.add_plugin(PickingPlugin)
			.add_plugin(DebugCursorPickingPlugin)
			//.add_plugin(DebugEventsPickingPlugin)
			.run();
	}
}

//////////////////////////////////////////////////////////////////////////////////////

fn set_title(time: Res<Time>, mut windows: ResMut<Windows>) {
	let window = windows.get_primary_mut().unwrap();
	window.set_title(format!("Seconds since startup: {}",time.seconds_since_startup().round()));
}

//////////////////////////////////////////////////////////////////////////////////////

fn listen_to_messages(
	mut commands: Commands,
	mut assets: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut mystate: ResMut<AWayToHaveGlobalState>
) {
    while let Ok(message) = mystate.receiver.try_recv() {
        match message {
            Message::Event(topic,data) => {
                println!("Graphics: Received: {} {}",topic, data);
                match data.as_str() {
                    "camera" => {
						commands.spawn_bundle(PerspectiveCameraBundle {
							transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
							..Default::default()
						})
					    .insert(OrbitCamera::default())
						.insert_bundle(PickingCameraBundle::default());
                    },
                    "light" => {
						commands.spawn_bundle(PointLightBundle {
							transform: Transform::from_xyz(4.0, 8.0, 4.0),
							..Default::default()
						});
                    },
                    "plane" => {
						commands.spawn_bundle(PbrBundle {
							mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
							material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
							..Default::default()
						});
                    },
                	"load" => {
                    },
                    "cube" => {
						commands.spawn_bundle(PbrBundle {
							mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
							material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
							transform: Transform::from_xyz(0.0, 0.5, 0.0),
							..Default::default()
						})
						
						.insert(MyProperties{ x:3.0, y:3.0 })
						.insert_bundle(PickableBundle::default());
                    },
                    "move" => {
                    },
                    _ => {
                    	// note meshes need vertex tangents (just use blender) -> https://github.com/bevyengine/bevy/issues/121
                    	println!("loading from disk");

						let path = "../../../public/".to_string() + data.as_str() + "#Scene0";

					    let stuff: Handle<Scene> = assets.load(path.as_str());
					    commands.spawn_scene(stuff);
/*

						let mesh: Handle<Mesh> = assets.load(path.as_str());
						commands.spawn_bundle(PbrBundle {
							mesh: mesh,
							material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
							//transform: Transform::from_xyz(0.0, 0.5, 0.0),
							transform: Transform::from_scale( Vec3::new(15.0,15.0,15.0) ),
							//transform: Scale::from_xyz(2.0,2.0,2.0),
							..Default::default()
						})
						//.with(MyProperties{ x:3.0, y:3.0 })
						.insert_bundle(PickableBundle::default());
*/

                    }
                }
            },
			_ => { },
		}
	}
}

fn move_things(
	time: Res<Time>,
	mut query: Query<(&mut Transform, &MyProperties)>
) {
    for (mut transform, myprops) in query.iter_mut() {
        // Get the direction to move in
        let direction = Vec3::new(myprops.x as f32, 0.0, myprops.y as f32) - transform.translation;
        // Only move if isn't already there (distance is big)
        if direction.length() > 0.1 {
            transform.translation += direction.normalize() * time.delta_seconds();
        }
    }
}

