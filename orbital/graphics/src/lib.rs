/*

Using bevy as a rendering layer:

It's not really designed to be used the way I am using it. There are several tensions:

- "system" -> bevy has a conceit that it itself should be your outermost scope or 'system'
- mainloop -> as a result it grabs the run loop; preventing you from doing any other work
- "resource" -> if you want to do work you have to carefully package up any state as a resource
- "system" -> bevy introduces a pile of concepts, one is a system that can "do work"
- "query" -> it's unclear exactly how a system gets arbitrary arguments but it just works

Unfortunately bevy itself is a way of thinking; and it has its own learning curve.

My approach:

- messages -> I've managed to get my message channel visible to a bevy "system" at runtime
- create -> I pass messages to my code to manufacture bevy objects as I wish

References:
	https://caballerocoll.com/blog/bevy-chess-tutorial/
	https://bevy-cheatbook.github.io/programming/res.html
	https://github.com/bevyengine/bevy/blob/main/examples/window/window_settings.rs
	https://github.com/bevyengine/bevy/blob/latest/examples/3d/3d_scene.rs
*/

//////////////////////////////////////////////////////////////////////////////////////

use std::fs;
use crossbeam::channel::*;
use service::*;

use bevy::prelude::*;
use bevy_mod_picking::*;

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

		App::build()
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
			.add_system(listen_to_messages.system())
			.add_system(move_things.system())
			.add_plugins(DefaultPlugins)
			.add_plugin(PickingPlugin)
			.add_plugin(DebugCursorPickingPlugin)
//			.add_plugin(DebugEventsPickingPlugin)
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
						.insert_bundle(PickingCameraBundle::default());
                    },
                    "light" => {
						commands.spawn_bundle(LightBundle {
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
						let path = "../../../public/".to_string() + data.as_str() + "#Mesh0/Primitive0";
						let mesh: Handle<Mesh> = assets.load(path.as_str());
						commands.spawn_bundle(PbrBundle {
							//mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
							mesh: mesh,
							material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
							transform: Transform::from_xyz(0.0, 0.5, 0.0),
							..Default::default()
						})
						//.with(MyProperties{ x:3.0, y:3.0 })
						.insert_bundle(PickableBundle::default());
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


/*

pub fn load_something (
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    position: Vec3,
) {
    commands
        .spawn(PbrBundle {
            transform: Transform::from_translation(position),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -0.95));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
        });
}


*/

