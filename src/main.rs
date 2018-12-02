#[macro_use]
extern crate bitflags;
extern crate cgmath;
extern crate ply_rs;
#[macro_use]
extern crate stdweb;
extern crate webgl;

use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;

use stdweb::unstable::TryInto;
use stdweb::web::{document, window, IEventTarget, IHtmlElement, IParentNode, TypedArray};

use stdweb::web::event::{IKeyboardEvent, KeyDownEvent, KeyUpEvent, ResizeEvent};

use stdweb::web::html_element::{CanvasElement, ImageElement};
use webgl::WebGLRenderingContext as gl;
use webgl::{WebGLBuffer, WebGLProgram, WebGLRenderingContext, WebGLUniformLocation};

use cgmath::{vec3, Deg, Euler, Matrix4, PerspectiveFov, Rad};

trait Mesh {
    fn vertices(&self) -> &[f32];
    fn normals(&self) -> &[f32];
    fn colors(&self) -> &[f32];
    fn indices(&self) -> &[u16];

    fn bind(&self, context: &WebGLRenderingContext) -> BoundMesh {
        let vertices = TypedArray::<f32>::from(self.vertices()).buffer();
        let vertex_buffer = context.create_buffer().unwrap();
        context.bind_buffer(gl::ARRAY_BUFFER, Some(&vertex_buffer));
        context.buffer_data_1(gl::ARRAY_BUFFER, Some(&vertices), gl::STATIC_DRAW);

        let normals = TypedArray::<f32>::from(self.normals()).buffer();
        let normal_buffer = context.create_buffer().unwrap();
        context.bind_buffer(gl::ARRAY_BUFFER, Some(&normal_buffer));
        context.buffer_data_1(gl::ARRAY_BUFFER, Some(&normals), gl::STATIC_DRAW);

        let colors = TypedArray::<f32>::from(self.colors()).buffer();
        let color_buffer = context.create_buffer().unwrap();
        context.bind_buffer(gl::ARRAY_BUFFER, Some(&color_buffer));
        context.buffer_data_1(gl::ARRAY_BUFFER, Some(&colors), gl::STATIC_DRAW);

        let indices = TypedArray::<u16>::from(self.indices()).buffer();
        let index_buffer = context.create_buffer().unwrap();
        context.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        context.buffer_data_1(gl::ELEMENT_ARRAY_BUFFER, Some(&indices), gl::STATIC_DRAW);

        BoundMesh::new(self.indices().len() as u16, vertex_buffer, normal_buffer, color_buffer, index_buffer)
    }
}

struct Cube;

impl Mesh for Cube {
    fn vertices(&self) -> &[f32] {
        &[
            -1., -1., -1., 1., -1., -1., 1., 1., -1., -1., 1., -1., -1., -1., 1., 1., -1., 1., 1.,
            1., 1., -1., 1., 1., -1., -1., -1., -1., 1., -1., -1., 1., 1., -1., -1., 1., 1., -1.,
            -1., 1., 1., -1., 1., 1., 1., 1., -1., 1., -1., -1., -1., -1., -1., 1., 1., -1., 1.,
            1., -1., -1., -1., 1., -1., -1., 1., 1., 1., 1., 1., 1., 1., -1.,
        ]
    }

    fn normals(&self) -> &[f32] {
        &[
            0., 0., -1., 0., 0., -1., 0., 0., -1., 0., 0., -1., 0., 0., 1., 0., 0., 1., 0., 0., 1.,
            0., 0., 1., -1., 0., 0., -1., 0., 0., -1., 0., 0., -1., 0., 0., 1., 0., 0., 1., 0., 0.,
            1., 0., 0., 1., 0., 0., 0., -1., 0., 0., -1., 0., 0., -1., 0., 0., -1., 0., 0., 1., 0.,
            0., 1., 0., 0., 1., 0., 0., 1., 0.,
        ]
    }

    fn colors(&self) -> &[f32] {
        &[
            0.8, 0.5, 1.0, 0.8, 0.5, 1.0, 0.8, 0.5, 1.0, 0.8, 0.5, 1.0, 0.2, 0.2, 0.5, 0.2, 0.2,
            0.5, 0.2, 0.2, 0.5, 0.2, 0.2, 0.5, 0., 0., 0.2, 0., 0., 0.2, 0., 0., 0.2, 0., 0., 0.2,
            0.2, 0., 0., 0.2, 0., 0., 0.2, 0., 0., 0.2, 0., 0., 0.2, 0.2, 0., 0.2, 0.2, 0., 0.2,
            0.2, 0., 0.2, 0.2, 0., 0., 0.2, 0., 0., 0.2, 0., 0., 0.2, 0., 0., 0.2, 0.,
        ]
    }

    fn indices(&self) -> &[u16] {
        &[
            0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16,
            17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
        ]
    }
}

struct PlyMesh {
    vertices: Vec<f32>,
    normals: Vec<f32>,
    colors: Vec<f32>,
    indices: Vec<u16>,
}

impl PlyMesh {
    pub fn parse<T: Read>(source: &mut T) -> Self {
        let parser = ply_rs::parser::Parser::<ply_rs::ply::DefaultElement>::new();
        let ply = parser.read_ply(source).unwrap();
        let vertices: Vec<f32> = ply.payload.get("vertex").unwrap()
            .iter()
            .flat_map(|x| match (x.get("x"), x.get("y"), x.get("z")) {
                (Some(ply_rs::ply::Property::Float(x)), Some(ply_rs::ply::Property::Float(y)), Some(ply_rs::ply::Property::Float(z))) => vec![x, y, z],
                _ => {
                    console!(log, "Something else");
                    vec![]
                }
            }).cloned()
            .collect();
        let normals: Vec<f32> = ply.payload.get("vertex").unwrap()
            .iter()
            .flat_map(|x| match (x.get("nx"), x.get("ny"), x.get("nz")) {
                (Some(ply_rs::ply::Property::Float(x)), Some(ply_rs::ply::Property::Float(y)), Some(ply_rs::ply::Property::Float(z))) => vec![x, y, z],
                _ => {
                    console!(log, "Something else");
                    vec![]
                }
            }).cloned()
            .collect();
        let colors: Vec<f32> = ply.payload.get("vertex").unwrap()
            .iter()
            .flat_map(|x| match (x.get("red"), x.get("green"), x.get("blue")) {
                (Some(ply_rs::ply::Property::UChar(r)), Some(ply_rs::ply::Property::UChar(g)), Some(ply_rs::ply::Property::UChar(b))) => vec![r, g, b],
                _ => {
                    console!(log, "Something else");
                    vec![]
                }
            }).map(|x| (*x as f32) / 255.)
            .collect();
        let indices: Vec<u16> = ply
            .payload
            .get("face")
            .unwrap()
            .iter()
            .filter_map(|x| match x.get("vertex_indices") {
                Some(ply_rs::ply::Property::ListUInt(x)) => Some(x),
                _ => {
                    console!(log, "Something else");
                    None
                }
            }).flat_map(|x| x.iter().map(|x| *x as u16))
            .collect();
        PlyMesh { vertices, normals, colors, indices }
    }
}

impl Mesh for PlyMesh {
    fn vertices(&self) -> &[f32] {
        self.vertices.as_slice()
    }
    fn normals(&self) -> &[f32] {
        self.normals.as_slice()
    }
    fn colors(&self) -> &[f32] {
        self.colors.as_slice()
    }
    fn indices(&self) -> &[u16] {
        self.indices.as_slice()
    }
}

struct BoundMesh {
    pub num_indices: u16,
    pub vertex_buffer: WebGLBuffer,
    pub normal_buffer: WebGLBuffer,
    pub color_buffer: WebGLBuffer,
    pub index_buffer: WebGLBuffer,
}

impl BoundMesh {
    pub fn new(
        num_indices: u16,
        vertex_buffer: WebGLBuffer,
        normal_buffer: WebGLBuffer,
        color_buffer: WebGLBuffer,
        index_buffer: WebGLBuffer,
    ) -> Self {
        BoundMesh {
            num_indices,
            vertex_buffer,
            normal_buffer,
            color_buffer,
            index_buffer,
        }
    }
}

struct Shader {
    pub program: WebGLProgram,
}

impl Shader {
    pub fn new(context: &WebGLRenderingContext, vertex_code: &str, fragment_code: &str) -> Self {
        let vertex_shader = context.create_shader(gl::VERTEX_SHADER).unwrap();
        context.shader_source(&vertex_shader, vertex_code);
        context.compile_shader(&vertex_shader);
        console!(log, context.get_shader_info_log(&vertex_shader));

        let fragment_shader = context.create_shader(gl::FRAGMENT_SHADER).unwrap();
        context.shader_source(&fragment_shader, fragment_code);
        context.compile_shader(&fragment_shader);
        console!(log, context.get_shader_info_log(&fragment_shader));

        let program = context.create_program().unwrap();
        context.attach_shader(&program, &vertex_shader);
        context.attach_shader(&program, &fragment_shader);
        context.link_program(&program);
        console!(log, context.get_program_info_log(&program));

        Shader { program }
    }
}

bitflags! {
    struct Keys: u8 {
        const UP    = 0b0000_0001;
        const DOWN  = 0b0000_0010;
        const LEFT  = 0b0000_0100;
        const RIGHT = 0b0000_1000;
    }
}

struct State {
    time_old: f64,
    mov_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    canvas: CanvasElement,
    context: WebGLRenderingContext,
    shader: Shader,
    position: u32,
    normal: u32,
    color: u32,
    p_matrix: WebGLUniformLocation,
    v_matrix: WebGLUniformLocation,
    m_matrix: WebGLUniformLocation,
    ziggurat: BoundMesh,
    peon: BoundMesh,
    keys: Keys,
    prev_keys: Keys,
}

impl State {
    fn animate(&mut self, time: f64, rc: Rc<RefCell<Self>>) {
        let dt = (time - self.time_old) as f32;
        self.mov_matrix = self.mov_matrix * Matrix4::<f32>::from(Euler::new(
            Rad(dt
                * 0.001
                * (self.keys.contains(Keys::UP) as i8 - self.keys.contains(Keys::DOWN) as i8)
                    as f32),
            Rad(dt
                * 0.001
                * (self.keys.contains(Keys::RIGHT) as i8 - self.keys.contains(Keys::LEFT) as i8)
                    as f32),
            Rad(0.),
        ));
        self.time_old = time;

        self.context.enable(gl::DEPTH_TEST);
        self.context.depth_func(gl::LEQUAL);
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear_depth(1.0);
        self.context.cull_face(gl::FRONT_AND_BACK);

        let (w, h) = (self.canvas.width(), self.canvas.height());
        let proj_matrix = PerspectiveFov {
            fovy: Deg(80.).into(),
            aspect: (w as f32) / (h as f32),
            near: 1.,
            far: 100.,
        };
        let proj_matrix: Matrix4<f32> = proj_matrix.into();

        self.context.viewport(0, 0, w as i32, h as i32);
        self.context
            .clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        self.context.use_program(Some(&self.shader.program));
        self.context.enable_vertex_attrib_array(self.position);
        self.context.enable_vertex_attrib_array(self.color);
        self.context.enable_vertex_attrib_array(self.normal);

        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&self.peon.vertex_buffer));
        self.context.vertex_attrib_pointer(self.position, 3, gl::FLOAT, false, 0, 0);

        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&self.peon.color_buffer));
        self.context.vertex_attrib_pointer(self.color, 3, gl::FLOAT, false, 0, 0);

        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&self.peon.normal_buffer));
        self.context.vertex_attrib_pointer(self.normal, 3, gl::FLOAT, false, 0, 0);

        self.context.uniform_matrix4fv(
            Some(&self.p_matrix),
            false,
            &(proj_matrix.as_ref() as &[f32; 16])[..],
        );
        self.context.uniform_matrix4fv(
            Some(&self.v_matrix),
            false,
            &(self.view_matrix.as_ref() as &[f32; 16])[..],
        );
        self.context.uniform_matrix4fv(
            Some(&self.m_matrix),
            false,
            &(self.mov_matrix.as_ref() as &[f32; 16])[..],
        );

        self.context
            .bind_buffer(gl::ELEMENT_ARRAY_BUFFER, Some(&self.peon.index_buffer));
        self.context
            .draw_elements(gl::TRIANGLES, self.peon.num_indices as i32, gl::UNSIGNED_SHORT, 0);

//        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&self.ziggurat.vertex_buffer));
//        self.context.vertex_attrib_pointer(self.position, 3, gl::FLOAT, false, 0, 0);
//
//        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&self.ziggurat.color_buffer));
//        self.context.vertex_attrib_pointer(self.color, 3, gl::FLOAT, false, 0, 0);
//
//        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&self.ziggurat.normal_buffer));
//        self.context.vertex_attrib_pointer(self.normal, 3, gl::FLOAT, false, 0, 0);
//
//        self.context.uniform_matrix4fv(
//            Some(&self.p_matrix),
//            false,
//            &(proj_matrix.as_ref() as &[f32; 16])[..],
//        );
//        self.context.uniform_matrix4fv(
//            Some(&self.v_matrix),
//            false,
//            &(self.view_matrix.as_ref() as &[f32; 16])[..],
//        );
//        self.context.uniform_matrix4fv(
//            Some(&self.m_matrix),
//            false,
//            &(self.mov_matrix.as_ref() as &[f32; 16])[..],
//        );
//
//        self.context
//            .bind_buffer(gl::ELEMENT_ARRAY_BUFFER, Some(&self.ziggurat.index_buffer));
//        self.context
//            .draw_elements(gl::TRIANGLES, self.ziggurat.num_indices as i32, gl::UNSIGNED_SHORT, 0);

        window().request_animation_frame(move |time| {
            rc.borrow_mut().animate(time, rc.clone());
        });
        self.prev_keys = self.keys;
    }
}

fn main() {
    stdweb::initialize();

    // Doesn't work because web-sys isn't integrated with stdweb.
    //let audio = web_sys::AudioContext::new().unwrap();
    //let source = audio.create_buffer_source().unwrap(); // creates a sound source
    ////source.buffer = buffer;                    // tell the source which sound to play
    //source.connect_with_audio_node(&audio.destination());       // connect the source to the context's destination (the speakers)
    //source.start();

    // Doesn't work due to https://github.com/koute/stdweb/issues/171
    //let image = ImageElement::new();
    //image.add_event_listener({
    //    let image = image.clone();
    //    move |evt: ResourceLoadEvent| {
    //        console!(log, "Image loaded");
    //        console!(log, "Image loaded", image.get_attribute("src"));
    //    }
    //});
    //image.add_event_listener({
    //    let image = image.clone();
    //    move |_: ResourceErrorEvent| {
    //        console!(log, "Image failed to load");
    //        console!(log, "Image failed to load", image.get_attribute("src"));
    //    }
    //});
    //image.add_event_listener({
    //    let image = image.clone();
    //    move |_: ResourceAbortEvent| {
    //        console!(log, "Image load aborted");
    //        console!(log, "Image load aborted", image.get_attribute("src"));
    //    }
    //});
    //image.set_src("test.png");

    let canvas: CanvasElement = document()
        .query_selector("#canvas")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();
    let context: WebGLRenderingContext = canvas.get_context().unwrap();

    canvas.set_width(canvas.offset_width() as u32);
    canvas.set_height(canvas.offset_height() as u32);

    window().add_event_listener({
        let canvas = canvas.clone();
        move |_: ResizeEvent| {
            canvas.set_width(canvas.offset_width() as u32);
            canvas.set_height(canvas.offset_height() as u32);
        }
    });

    let peon_ply = include_str!("../models/peon.ply");
    let peon = PlyMesh::parse(&mut peon_ply.as_bytes()).bind(&context);

    let ziggurat_ply = include_str!("../models/ziggurat.ply");
    let ziggurat = PlyMesh::parse(&mut ziggurat_ply.as_bytes()).bind(&context);

    let shader = Shader::new(
        &context,
        r#"
            attribute vec3 position;
            attribute vec3 normal;
            uniform mat4 Pmatrix;
            uniform mat4 Vmatrix;
            uniform mat4 Mmatrix;
            attribute vec3 color;
            varying vec3 vColor;
            varying vec3 vNormal;
            varying vec3 vFragPos;

            void main() {
                vFragPos = vec3(Mmatrix * vec4(position, 1.));
                gl_Position = Pmatrix*Vmatrix*vec4(vFragPos, 1.);
                vNormal = vec3(Mmatrix * vec4(normal, 1.));
                vColor = color;
            }
        "#,
        r#"
            precision mediump float;
            varying vec3 vColor;
            varying vec3 vNormal;
            varying vec3 vFragPos;

            void main() {
                float diffuse = max(dot(vNormal, normalize(vec3(0., 0., 6.) - vFragPos)), 0.0);
                gl_FragColor = vec4(vColor * (0.5 + 0.5 * diffuse), 1.0);
            }
        "#,
    );

    /* ====== Associating attributes to vertex shader =====*/
    let p_matrix = context
        .get_uniform_location(&shader.program, "Pmatrix")
        .unwrap();
    let v_matrix = context
        .get_uniform_location(&shader.program, "Vmatrix")
        .unwrap();
    let m_matrix = context
        .get_uniform_location(&shader.program, "Mmatrix")
        .unwrap();

    let position = context.get_attrib_location(&shader.program, "position") as u32;
    let color = context.get_attrib_location(&shader.program, "color") as u32;
    let normal = context.get_attrib_location(&shader.program, "normal") as u32;

    let state = Rc::new(RefCell::new(State {
        time_old: 0.0,
        mov_matrix: Matrix4::from_scale(1.),
        view_matrix: Matrix4::from_translation(vec3(0., 0., -6.)),
        canvas,
        context,
        shader,
        position,
        color,
        normal,
        p_matrix,
        v_matrix,
        m_matrix,
        ziggurat,
        peon,
        keys: Keys::empty(),
        prev_keys: Keys::empty(),
    }));

    window().add_event_listener({
        let state = state.clone();
        move |evt: KeyDownEvent| match evt.code().as_str() {
            "KeyA" => state.borrow_mut().keys |= Keys::LEFT,
            "KeyW" => state.borrow_mut().keys |= Keys::UP,
            "KeyS" => state.borrow_mut().keys |= Keys::DOWN,
            "KeyD" => state.borrow_mut().keys |= Keys::RIGHT,
            _ => {}
        }
    });

    window().add_event_listener({
        let state = state.clone();
        move |evt: KeyUpEvent| match evt.code().as_str() {
            "KeyA" => state.borrow_mut().keys &= !Keys::LEFT,
            "KeyW" => state.borrow_mut().keys &= !Keys::UP,
            "KeyS" => state.borrow_mut().keys &= !Keys::DOWN,
            "KeyD" => state.borrow_mut().keys &= !Keys::RIGHT,
            _ => {}
        }
    });

    state.borrow_mut().animate(0., state.clone());

    stdweb::event_loop();
}
