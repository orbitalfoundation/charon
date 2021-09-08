use crate::cx::*;

#[derive(Default, Clone)]
pub struct Pass {
    pub pass_id: Option<usize>
}


impl Pass {
    pub fn begin_pass(&mut self, cx: &mut Cx) {
        
        if self.pass_id.is_none() { // we need to allocate a CxPass
            self.pass_id = Some(if cx.passes_free.len() != 0 {
                cx.passes_free.pop().unwrap()
            } else {
                cx.passes.push(CxPass::default());
                cx.passes.len() - 1
            });
        }
        let pass_id = self.pass_id.unwrap();
        
        if let Some(window_id) = cx.window_stack.last() {
            if cx.windows[*window_id].main_pass_id.is_none() { // we are the main pass of a window
                let cxpass = &mut cx.passes[pass_id];
                cx.windows[*window_id].main_pass_id = Some(pass_id);
                cxpass.dep_of = CxPassDepOf::Window(*window_id);
                cxpass.pass_size = cx.windows[*window_id].get_inner_size();
                cx.current_dpi_factor = cx.get_delegated_dpi_factor(pass_id);
            }
            else if let Some(dep_of_pass_id) = cx.pass_stack.last() { 
                let dep_of_pass_id = *dep_of_pass_id;
                cx.passes[pass_id].dep_of = CxPassDepOf::Pass(dep_of_pass_id);
                cx.passes[pass_id].pass_size = cx.passes[dep_of_pass_id].pass_size;
                cx.current_dpi_factor = cx.get_delegated_dpi_factor(dep_of_pass_id);
            }
            else {
                cx.passes[pass_id].dep_of = CxPassDepOf::None;
                cx.passes[pass_id].override_dpi_factor = Some(1.0);
                cx.current_dpi_factor = 1.0;
            }
        }
        else {
            cx.passes[pass_id].dep_of = CxPassDepOf::None;
            cx.passes[pass_id].override_dpi_factor = Some(1.0);
            cx.current_dpi_factor = 1.0;
        }
        
        let cxpass = &mut cx.passes[pass_id];
        cxpass.main_view_id = None;
        cxpass.color_textures.truncate(0);
        cx.pass_stack.push(pass_id);
        
        //let pass_size = cxpass.pass_size;
        //self.set_ortho_matrix(cx, Vec2::zero(), pass_size);
    }
    
    pub fn override_dpi_factor(&mut self, cx: &mut Cx, dpi_factor:f32){
        if let Some(pass_id) = self.pass_id {
            cx.passes[pass_id].override_dpi_factor = Some(dpi_factor);
            cx.current_dpi_factor = dpi_factor;
        }
    }
    
    pub fn make_dep_of_pass(&mut self, cx: &mut Cx, pass: &Pass) {
        let cxpass = &mut cx.passes[self.pass_id.unwrap()];
        if let Some(pass_id) = pass.pass_id {
            cxpass.dep_of = CxPassDepOf::Pass(pass_id)
        }
        else {
            cxpass.dep_of = CxPassDepOf::None
        }
    }
    
    pub fn set_size(&mut self, cx: &mut Cx, pass_size: Vec2) {
        let mut pass_size = pass_size;
        if pass_size.x < 1.0{pass_size.x = 1.0};
        if pass_size.y < 1.0{pass_size.y = 1.0};
        let cxpass = &mut cx.passes[self.pass_id.unwrap()];
        cxpass.pass_size = pass_size;
    }
    
    pub fn set_window_clear_color(&mut self, cx: &mut Cx, clear_color: Vec4) {
        let pass_id = self.pass_id.expect("Please call set_window_clear_color after begin_pass");
        let cxpass = &mut cx.passes[pass_id];
        cxpass.clear_color = clear_color;
    }
    
    pub fn add_color_texture(&mut self, cx: &mut Cx, texture: Texture, clear_color: ClearColor) {
        let pass_id = self.pass_id.expect("Please call add_color_texture after begin_pass");
        let cxpass = &mut cx.passes[pass_id];
        cxpass.color_textures.push(CxPassColorTexture {
            texture_id: texture.texture_id,
            clear_color: clear_color
        })
    }
    
    pub fn set_depth_texture(&mut self, cx: &mut Cx, texture: Texture, clear_depth: ClearDepth) {
        let pass_id = self.pass_id.expect("Please call set_depth_texture after begin_pass");
        let cxpass = &mut cx.passes[pass_id];
        cxpass.depth_texture = Some(texture.texture_id);
        cxpass.clear_depth = clear_depth;
    }
    
    pub fn set_matrix_mode(&mut self, cx: &mut Cx, pmm: PassMatrixMode){
        if let Some(pass_id) = self.pass_id{
            let cxpass = &mut cx.passes[pass_id];
            cxpass.paint_dirty = true;
            cxpass.matrix_mode = pmm;
        }
    }

    pub fn set_debug(&mut self, cx: &mut Cx, debug:bool){
        if let Some(pass_id) = self.pass_id{
            let cxpass = &mut cx.passes[pass_id];
            cxpass.debug = debug;
        }
    }

    
    pub fn end_pass(&mut self, cx: &mut Cx) {
        cx.pass_stack.pop();
        if cx.pass_stack.len()>0{
            cx.current_dpi_factor = cx.get_delegated_dpi_factor(*cx.pass_stack.last().unwrap());
        }
    }
    
    pub fn redraw_pass_area(&mut self, cx: &mut Cx) {
        if let Some(pass_id) = self.pass_id {
            cx.redraw_pass_and_sub_passes(pass_id);
        }
    }
    
}

#[derive(Clone)]
pub enum ClearColor {
    InitWith(Vec4),
    ClearWith(Vec4)
}

impl Default for ClearColor {
    fn default() -> Self {
        ClearColor::ClearWith(Vec4::default())
    }
}

#[derive(Clone)]
pub enum ClearDepth {
    InitWith(f64),
    ClearWith(f64)
}

#[derive(Default, Clone)]
pub struct CxPassColorTexture {
    pub clear_color: ClearColor,
    pub texture_id: u32
}

#[derive(Default, Clone)]
#[repr(C)]
pub struct PassUniforms{
    camera_projection:[f32;16],
    camera_view:[f32;16],
    camera_inv:[f32;16],
    dpi_factor:f32,
    dpi_dilate:f32,
    pad1:f32,
    pad2:f32
}

impl PassUniforms{
    pub fn as_slice(&self)->&[f32;std::mem::size_of::<PassUniforms>()]{
        unsafe{std::mem::transmute(self)}
    }
}

#[derive(Clone)]
pub enum PassMatrixMode{
    Ortho,
    Projection{fov_y:f32, near:f32, far:f32, cam:Mat4}
}

#[derive(Clone)]
pub struct CxPass {
    pub debug: bool,
    pub matrix_mode: PassMatrixMode,
    pub color_textures: Vec<CxPassColorTexture>,
    pub depth_texture: Option<u32>,
    pub clear_depth: ClearDepth,
    pub depth_init: f64,
    pub clear_color: Vec4,
    pub override_dpi_factor: Option<f32>,
    pub main_view_id: Option<usize>,
    pub dep_of: CxPassDepOf,
    pub paint_dirty: bool,
    pub pass_size: Vec2,
    pub pass_uniforms: PassUniforms,
    pub zbias_step: f32,
    pub platform: CxPlatformPass,
}

impl Default for CxPass {
    fn default() -> Self {
        CxPass {
            debug: false,
            matrix_mode: PassMatrixMode::Ortho,
            zbias_step: 0.001,
            pass_uniforms: PassUniforms::default(),
            color_textures: Vec::new(),
            depth_texture: None,
            override_dpi_factor: None,
            clear_depth: ClearDepth::ClearWith(1.0),
            clear_color: Vec4::default(),
            depth_init: 1.0,
            main_view_id: None,
            dep_of: CxPassDepOf::None,
            paint_dirty: false,
            pass_size: Vec2::default(),
            platform: CxPlatformPass::default()
        }
    }
}

#[derive(Clone, Debug)]
pub enum CxPassDepOf {
    Window(usize),
    Pass(usize),
    None
}


impl CxPass {
    
    pub fn uniform_camera_projection(&mut self, v: &Mat4) {
        //dump in uniforms
        for i in 0..16 {
            self.pass_uniforms.camera_projection[i] = v.v[i];
        }
    }
    
    pub fn uniform_camera_view(&mut self, v: &Mat4) {
        //dump in uniforms
        for i in 0..16 {
            self.pass_uniforms.camera_view[i] =  v.v[i];
        }
    }
    
    pub fn set_dpi_factor(&mut self, dpi_factor: f32) {
        let dpi_dilate = (2. - dpi_factor).max(0.).min(1.);
        self.pass_uniforms.dpi_factor = dpi_factor;
        self.pass_uniforms.dpi_dilate = dpi_dilate;
    }
    
    pub fn set_matrix(&mut self, offset: Vec2, size: Vec2) {
         match self.matrix_mode{
            PassMatrixMode::Ortho=>{
                let ortho = Mat4::ortho(
                    offset.x,
                    offset.x + size.x,
                    offset.y,
                    offset.y + size.y,
                    100.,
                    -100.,
                    1.0,
                    1.0
                );
                self.uniform_camera_projection(&ortho);
                self.uniform_camera_view(&Mat4::identity());
            }
            PassMatrixMode::Projection{fov_y, near, far, cam}=>{
                let proj = Mat4::perspective(fov_y, size.x / size.y, near, far);
                self.uniform_camera_projection(&proj);
                self.uniform_camera_view(&cam);
                
                /*Mat4::ortho(
                    offset.x,
                    offset.x + size.x,
                    offset.y,
                    offset.y + size.y,
                    100.,
                    -100.,
                    1.0,
                    1.0)*/
            }
        };
        //self.set_matrix(cx, &ortho_matrix);
    }
    
    //pub fn set_matrix(&mut self, cx: &mut Cx, matrix: &Mat4) {
    //let pass_id = self.pass_id.expect("Please call set_ortho_matrix after begin_pass");
    //let cxpass = &mut cx.passes[pass_id];
    // }
}