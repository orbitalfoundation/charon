
/*
Browser desktop / main window - apr 2021 thoughts
This is a sketch of what I'm imagining the browser main window will look like and how it will be driven.
There are several ideas that need to be evaluated:
    - userland apps are going to want to open and paint to a view - they should be able to follow well established conventions
        in the case of the web for example there's an assumption each app having one window or surface, driven by the DOM, and one paints into a canvas
        i can see an argument for a more flexible approach where userland apps can ask to produce a window and then paint directly into that
        there is also an idea of all the apps being accessible as separate pages (this is a common browser pattern and in a sense on ios as well)
    - the desktop itself is just another app similar to userland apps but it has a slightly privileged role
        a portion of it cannot be painted over by userland apps (for security reasons)
        it offers a hypervisor like ability to enumerate all other apps, get detailed performance and permissions on them, and start/stop them
        it has a control input box of some kind that is the users first point of entry to the system; similar to a browser URL input bar
        note that apps are persistent; unlike a browser, they keep running forever
    - userland apps may even paint to the _same_ view; an AR app has a need to have a single view
        this also means that apps may need some awareness of what other apps are painting to arbitrate visual collisions
        also there's an argument for a computer vision kind of semantic partitioning capability so that apps can have some spatial understanding
    - userland apps are going to want a well established grammar or dsl or some kind of formalization of how to paint
        it makes sense to require userland apps to directly ship a shader to the display layer to do work; it's a well established pattern
        but it may also make sense to offer higher level abstractions as well; even complex objects such as glb or the like
        in that sense this becomes more like a video game engine; it (optionally) offers a high level DAG and a DSL and a set of conventions...
    - the biggest challenge is in fact devising the right grammar abstraction
        - how much of this do we need to provide and what can be pushed to higher level third party intermediaries? what existing stuff can be used?
        - express vertex and pixel shaders in a standards based way; i'd like to expose the live shader idea that makepad has
        - shaders: exposing basically raw shaders
        - primitives: 2d colored boxes, fonts, lines, textured surfaces, blit copy regions, pipe rendering flows together
        - 3d primitives: geometry, constraint based physics, framebuffer effects, geometry extrusions ; this may be in userland (not our job)
            - see 3js
            - see babylon3d
            - see https://guide.nannou.cc/why_nannou.html
            - see https://bevyengine.org/
            - see https://rapier.rs/
        - text input boxes, buttons, scrollbars, divs, layout helpers (see https://github.com/hecrj/iced)
*/


////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// here is some glue code to present this desktop as a userland app for the kernel
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

use crossbeam::channel::*;

use service::*;

use makepad_render::*;
use makepad_widget::*;

#[derive(Clone)]
pub struct ViewMakepad {
}
impl ViewMakepad {
	pub fn new() -> Box<dyn Serviceable> {
		Box::new(Self {})
	}
}
impl Serviceable for ViewMakepad {
    fn name(&self) -> &str { "ViewMakepad" }
	fn stop(&self) {}
	fn start(&self, _name: String, sid: SID, send: Sender<Message>, recv: Receiver<Message> ) {

        // listen to display messages
		send.send(Message::Subscribe(sid,"/view".to_string())).expect("ViewMakepad: failed to subscribe");

        // open a display -> this never returns for now!!!
        let mut cx = Cx::default();
        cx.style();
        OrbitalBrowserDesktopUX::style(&mut cx);
        cx.init_live_styles();
        let mut app = OrbitalBrowserDesktopUX::new(&mut cx,send,recv);
        let mut cxafterdraw = CxAfterDraw::new(&mut cx);
        cx.event_loop( | cx, mut event | {
            if let Event::Draw = event {
                app.draw_app(cx);
                cxafterdraw.after_draw(cx);
                return
            }
            app.handle_app(cx, &mut event);
        });


	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// here we jump over to makepad to do real work - this is all just scratch test code right now
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// extern crate rustface;
// use rustface::{Detector, ImageData};
//const BUFSIZE : usize = 1280*720/4;

pub struct OrbitalBrowserDesktopUX {
    desktop_window: DesktopWindow, 
    menu: Menu,
    draw_image: DrawImage,
    image_texture: Texture,
    world_view: WorldView,
    textinput:TextInput,
    button:NormalButton,
    send:Sender<Message>,
    recv:Receiver<Message>,
    //detector:Box<dyn Detector>,
    //buffer:Box<[u8;BUFSIZE]>,
}

impl OrbitalBrowserDesktopUX {
    pub fn new(cx: &mut Cx, send: Sender<Message>, recv: Receiver<Message>) -> Self {

        let mut texture = Texture::new(cx);
        texture.set_desc(cx, TextureDesc{
            format: TextureFormat::ImageBGRA,
            width:Some(1280),
            height:Some(720),
            multisample: None
        });
        let cxtexture = &mut cx.textures[texture.texture_id as usize];
        cxtexture.image_u32.resize(1280*720,0);

        //let mut detector = rustface::create_detector("../public/resources/seeta_fd_frontal_v1.0.bin").unwrap();
        //detector.set_min_face_size(20);
        //detector.set_score_thresh(2.0);
        //detector.set_pyramid_scale_factor(0.8);
        //detector.set_slide_window_step(4, 4);
        //println!("loaded face detector");

        //let buffer = Box::new([0u8;BUFSIZE]);

        Self {
            desktop_window: DesktopWindow::new(cx).with_inner_layout(Layout{
                line_wrap: LineWrap::NewLine,
                ..Layout::default()
            }),
            menu:Menu::main(vec![
                Menu::sub("Example", vec![
                    Menu::line(),
                    Menu::item("Quit Example",  Cx::command_quit()),
                ]),
            ]),
            draw_image: DrawImage::new(cx, default_shader!()),
            world_view: WorldView::new(cx),
            image_texture: texture,
            textinput: TextInput::new(cx,TextInputOptions { multiline:false, read_only: false, empty_message: "Enter URL here".to_string() }),
            button: NormalButton::new(cx),
            send:send,
            recv:recv,
            //detector:detector,
            //buffer:buffer,
        }
    }
    
    pub fn style(cx: &mut Cx){
        set_widget_style(cx);
        WorldView::style(cx);
        SkyBox::style(cx);
    }
       
    pub fn handle_app(&mut self, cx: &mut Cx, event: &mut Event) {

        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // draw primitives
        while let Ok(message) = self.recv.try_recv() {
            match message {
                Message::Event(topic,data) => {
                    println!("Display: Received: {} {}",topic, data);
                    match data.as_str() {
                        "cube" => {
                            let thing = SceneThing { x:0.0, y:0.0, s:0.0, kind:1};
                            self.world_view.add( thing );
                        },
                        _ => {

                            let thing = SceneThing { x:0.0, y:0.0, s:0.0, kind:2};
                            self.world_view.add( thing );

                        }
                    }
 
                },
                Message::Share(sharedmemory) => {

                    // paint to texture
                    let texture = self.image_texture;
                    let cxtexture = &mut cx.textures[texture.texture_id as usize];
                    let mut ptr = sharedmemory.lock().unwrap();
                    for y in 0..720{
                        for x in 0..1280{
                            let pixel = ptr[y*1280+x];
                            let pixel = pixel.swap_bytes().rotate_right(8);  // target format is ARGB ignoring A, and src format is probaby RGBA
                            cxtexture.image_u32[y*1280+x]=pixel;
                        }
                    }
                    cxtexture.update_image = true;

// FACE DETECTOR IS TOO SLOW -> put on separate thread?

/*



    for y in 0..360{
        for x in 0..640{
            let pixel = ptr[y*1280*2+x*2];
            //let pixel = pixel.swap_bytes(); //.rotate_right(8);  // target format is ARGB ignoring A, and src format is probaby RGBA
            let pixel = pixel as u8;
            self.buffer[y*640+x]=pixel;
        }
    }
    let mut image = ImageData::new(self.buffer.as_mut(), 640, 360);

    for face in self.detector.detect(&mut image).into_iter() {
        let x = 2 * face.bbox().x() as usize;
        let y = 2 * face.bbox().y() as usize;
        let w = 2 * face.bbox().width() as usize;
        let h = 2 * face.bbox().height() as usize;

        if h < 20 { break; }
        if w < 20 { break; }
        if w > 400 { break; }
        if h > 400 { break; }
        if y < 40 { break };
        if y + h > 700 { break; }
        if x < 40 { break; }
        if x + w > 1240 { break }
  
        for i in 0 .. w {
            cxtexture.image_u32[y*1280+x+i]=0xff00ff00;
            cxtexture.image_u32[y*1280+x+i+1280]=0xff00ff00;
            cxtexture.image_u32[y*1280+x+i+1280+1280]=0xff00ff00;

            cxtexture.image_u32[(y+h)*1280+x+i]=0xff00ff00;
            cxtexture.image_u32[(y+h)*1280+x+i+1280]=0xff00ff00;
            cxtexture.image_u32[(y+h)*1280+x+i+1280+1280]=0xff00ff00;
        }

        for i in 0 .. h {
            cxtexture.image_u32[(y+i)*1280+x]=0xff00ff00;
            cxtexture.image_u32[(y+i)*1280+x+1]=0xff00ff00;
            cxtexture.image_u32[(y+i)*1280+x+2]=0xff00ff00;

            cxtexture.image_u32[(y+i)*1280+x+w]=0xff00ff00;
            cxtexture.image_u32[(y+i)*1280+x+w+1]=0xff00ff00;
            cxtexture.image_u32[(y+i)*1280+x+w+2]=0xff00ff00;
        }

    }
*/


                }
                _ => { },
            }
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////

        self.desktop_window.handle_desktop_window(cx, event);

        if let TextEditorEvent::KeyFocusLost = self.textinput.handle_text_input(cx,event) {
            // TODO - detect carriage return
        }

        if let ButtonEvent::Clicked = self.button.handle_normal_button(cx,event) {
            let str = self.textinput.get_value();
            println!("User has asked to load this url: {}",str);
//TODO            let _ = self.send.send(Message::BrokerGoto(str));
        }

        self.world_view.handle_world_view(cx, event);        
    }
    
    pub fn draw_app(&mut self, cx: &mut Cx) {
        if self.desktop_window.begin_desktop_window(cx, Some(&self.menu)).is_err() {
            return
        };
        self.world_view.draw_world_view_2d(cx);
            
        cx.reset_turtle_pos();

        self.button.draw_normal_button(cx, "GO");

        self.textinput.draw_text_input(cx);

        if true {
            self.draw_image.texture = self.image_texture.into();
            self.draw_image.draw_quad_abs(cx, Rect{pos:Vec2{x:100.0,y:100.0},size:Vec2{x:356.0,y:200.0}});
        }

        self.desktop_window.end_desktop_window(cx);
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// 3d immediate mode test
//
// this is throwaway code, here i'm exploring these ideas
//
//  - should there be a scenegraph representation? i'd prefer to avoid this -> prefer to require userland apps to pass shaders
//  - but if there is; what are reasonable scene graph nodes?
//  - what if multiple apps want to paint to the same window? whats a good collision avoidance policy?
//
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct SceneThing {
    pub x: f64,
    pub y: f64,
    pub s: f64,
    pub kind: u32,
}

#[derive(Clone)]
pub struct WorldView {
    pub view: View,
    pub time: f64,
    pub sky_box: SkyBox,
    pub cube: DrawCube,
    pub viewport_3d: Viewport3D,
    pub next_frame: NextFrame,
    pub scene: Vec<SceneThing>,
}

impl WorldView {

    pub fn new(cx: &mut Cx) -> Self {
        Self {
            view: View::new(),
            time: 0.0,
            viewport_3d: Viewport3D::new(cx),
            next_frame: NextFrame::default(),
            sky_box: SkyBox::new(cx),
            cube: DrawCube::new(cx, default_shader!()),
            scene: Vec::new(),
        }
    }
    
    pub fn add(&mut self, x: SceneThing) {
        self.scene.push(x);
    }

    pub fn style(cx: &mut Cx) {
        live_body!(cx, r#"
            self::color_bg: #222222;
        "#);
    }

    pub fn handle_world_view(&mut self, cx: &mut Cx, event: &mut Event) {
        self.viewport_3d.handle_viewport_2d(cx, event);        
        if let Some(ae) = event.is_next_frame(cx, self.next_frame) {
            self.time = ae.time;
            self.view.redraw_view(cx);
        } 
    }
    
    pub fn draw_world_view_2d(&mut self, cx: &mut Cx) {        
        if self.viewport_3d.begin_viewport_3d(cx).is_ok() {
            self.draw_world_view_3d(cx);
            self.viewport_3d.end_viewport_3d(cx)
        };
        self.viewport_3d.draw_viewport_2d(cx);
    }
    
    pub fn draw_world_view_3d(&mut self, cx: &mut Cx) {

        if self.view.begin_view(cx, Layout::abs_origin_zero()).is_err() {
            return
        };
        
        self.view.lock_view_transform(cx, &Mat4::identity());
        
        self.sky_box.draw_sky_box(cx);

        // in this hack i'm pretending i have a scene
        // and then i'm painting what the scene says
        // ideally i'd only paint the active threads; which the user can select

        for x in &self.scene {

            match x.kind {
                1 => {

                    let mut cube2 = DrawCube::new(cx, default_shader!());

                    let mat = Mat4::txyz_s_ry_rx_txyz(
                        Vec3{x:0.0,y:0.0,z:0.0},
                        1.0,0.0,0.0,
                        Vec3{x:0.0, y:0.5, z:-1.5}
                    );
                    cube2.transform = mat;
                    cube2.cube_size=Vec3{x:0.05, y:0.05, z:0.05 };
                    cube2.cube_pos=Vec3{x:0.05,y:0.05,z:0.05};
                    cube2.set_color(cx, Vec4{x:1.0, y:1.0,z:0.0, w:1.0});
                    cube2.draw_cube(cx);

                },
                2 => {

                    for i in 0..2000{
                        let ti = (i as f32)/4.0 + (self.time*0.1) as f32;
                        let mat = Mat4::txyz_s_ry_rx_txyz(
                            Vec3{x:0.0,y:0.0,z:0.0},
                            1.0,
                            (self.time*15.0) as f32 + ti*10.,(self.time*15.0) as f32,
                            Vec3{x:0.0, y:0.5, z:-1.5} 
                        );
                        self.cube.transform = mat; 
                        self.cube.cube_size = Vec3{x:0.05,y:0.05,z:0.05};
                        self.cube.cube_pos = Vec3{x:ti.sin()*0.8,y:ti.cos()*0.8,z:(ti*3.0).cos()*0.8};
                        self.cube.draw_cube(cx);
                    }

                },
                3 => {

                }
                _ => {

                }
            }
        }
       
        self.view.end_view(cx,);
        self.next_frame = cx.new_next_frame();
    }
    
} 



#[derive(Clone)]
pub struct SkyBox {
    cube: DrawCube,
}

impl SkyBox {
    pub fn new(cx: &mut Cx) -> Self {
        Self {
            cube: DrawCube::new(cx, live_shader!(cx, self::shader_sky_box))
        }
    }
    
    pub fn style(cx: &mut Cx) {
        live_body!(cx, r#"
            self::sky_color: #000000;
            self::edge_color: #111111;
            self::floor_color: #8;
            self::size: vec3(200.0, 100.0, 200.0);
            self::pos: vec3(0.0, 50.0, 0.);
            
            self::shader_sky_box: Shader {
                use makepad_render::drawcube::shader::*;
                fn color_form_id() -> vec4 {
                    if geom_id>4.5 {
                        return #f00;
                    }
                    if geom_id>3.5 {
                        return #0f0;
                    }
                    if geom_id>2.5 {
                        return #00f;
                    }
                    if geom_id>1.5 {
                        return #0ff;
                    }
                    return #f0f;
                }
                varying t:float;
                fn vertex() -> vec4 {
                
                    let model_view = camera_view * view_transform * transform ;
                    return camera_projection * (model_view * vec4(
                        geom_pos.x * cube_size.x + cube_pos.x,
                        geom_pos.y * cube_size.y + cube_pos.y,
                        geom_pos.z * cube_size.z + cube_pos.z + draw_zbias,
                        1.
                    ));
                }
                
                fn pixel() -> vec4 { 
                    let x = geom_uv.x;
                    let y = geom_uv.y;
                    // walls
                    let sky = self::sky_color;
                    let edge = self::edge_color;
                    if geom_id>4.5 || geom_id > 3.5 || geom_id < 1.5 {
                        return mix(edge, sky, y);
                    }
                    // floor
                    if geom_id>2.5 {
                        let coord = geom_uv * 150.0;
                        let grid = abs(
                            fract(coord - 0.5) - 0.5
                        ) / (abs(dFdx(coord)) + abs(dFdy(coord)));
                        let line = min(grid.x, grid.y);
                        let grid2 = self::floor_color + 0.4 * vec4(vec3(1.0 - min(line, 1.0)), 1.0);
                        let uv2 = abs(2.0 * geom_uv - 1.0);
                        return mix(grid2, edge, min(max(uv2.x, uv2.y) + 0.7, 1.0));
                    }
                    return sky;
                }
            }
        "#)
    }
    
    pub fn draw_sky_box(&mut self, cx: &mut Cx) {
        self.cube.cube_size = live_vec3!(cx, self::size);
        self.cube.cube_pos = live_vec3!(cx, self::pos);
        self.cube.draw_cube(cx);
    }
}
