extern crate cgmath;
extern crate stdweb;
extern crate webgl;

use std::cell::RefCell;
use std::rc::Rc;

use stdweb::unstable::TryInto;
use stdweb::web::{document, window, IEventTarget, IHtmlElement, IParentNode, TypedArray};

use stdweb::web::event::ResizeEvent;

use stdweb::web::html_element::CanvasElement;
use webgl::WebGLRenderingContext as gl;
use webgl::{
    WebGLBuffer, WebGLProgram, WebGLRenderingContext, WebGLUniformLocation,
};

use cgmath::{vec3, Deg, Euler, Matrix4, PerspectiveFov, Rad};

trait Mesh {
    fn vertices(&self) -> &[f32];
    fn colors(&self) -> &[f32];
    fn indices(&self) -> &[u16];

    fn bind(&self, context: &WebGLRenderingContext) -> BoundMesh {
        let vertices = TypedArray::<f32>::from(self.vertices()).buffer();
        let vertex_buffer = context.create_buffer().unwrap();
        context.bind_buffer(gl::ARRAY_BUFFER, Some(&vertex_buffer));
        context.buffer_data_1(gl::ARRAY_BUFFER, Some(&vertices), gl::STATIC_DRAW);

        let colors = TypedArray::<f32>::from(self.colors()).buffer();
        let color_buffer = context.create_buffer().unwrap();
        context.bind_buffer(gl::ARRAY_BUFFER, Some(&color_buffer));
        context.buffer_data_1(gl::ARRAY_BUFFER, Some(&colors), gl::STATIC_DRAW);

        let indices = TypedArray::<u16>::from(self.indices()).buffer();
        let index_buffer = context.create_buffer().unwrap();
        context.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        context.buffer_data_1(gl::ELEMENT_ARRAY_BUFFER, Some(&indices), gl::STATIC_DRAW);

        BoundMesh::new(vertex_buffer, color_buffer, index_buffer)
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

    fn colors(&self) -> &[f32] {
        &[
            5., 3., 7., 5., 3., 7., 5., 3., 7., 5., 3., 7., 1., 1., 3., 1., 1., 3., 1., 1., 3., 1.,
            1., 3., 0., 0., 1., 0., 0., 1., 0., 0., 1., 0., 0., 1., 1., 0., 0., 1., 0., 0., 1., 0.,
            0., 1., 0., 0., 1., 1., 0., 1., 1., 0., 1., 1., 0., 1., 1., 0., 0., 1., 0., 0., 1., 0.,
            0., 1., 0., 0., 1., 0.,
        ]
    }

    fn indices(&self) -> &[u16] {
        &[
            0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16,
            17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
        ]
    }
}

struct BoundMesh {
    pub vertex_buffer: WebGLBuffer,
    pub color_buffer: WebGLBuffer,
    pub index_buffer: WebGLBuffer,
}

impl BoundMesh {
    pub fn new(
        vertex_buffer: WebGLBuffer,
        color_buffer: WebGLBuffer,
        index_buffer: WebGLBuffer,
    ) -> Self {
        BoundMesh {
            vertex_buffer,
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

        let fragment_shader = context.create_shader(gl::FRAGMENT_SHADER).unwrap();
        context.shader_source(&fragment_shader, fragment_code);
        context.compile_shader(&fragment_shader);

        let program = context.create_program().unwrap();
        context.attach_shader(&program, &vertex_shader);
        context.attach_shader(&program, &fragment_shader);
        context.link_program(&program);

        Shader { program }
    }
}

struct State {
    time_old: f64,
    mov_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    canvas: CanvasElement,
    context: WebGLRenderingContext,
    p_matrix: WebGLUniformLocation,
    v_matrix: WebGLUniformLocation,
    m_matrix: WebGLUniformLocation,
    cube: BoundMesh,
}

impl State {
    fn animate(&mut self, time: f64, rc: Rc<RefCell<Self>>) {
        let dt = (time - self.time_old) as f32;
        self.mov_matrix = self.mov_matrix * Matrix4::<f32>::from(Euler::new(
            Rad(dt * 0.0002),
            Rad(dt * 0.0003),
            Rad(dt * 0.0007),
        ));
        self.time_old = time;

        self.context.enable(gl::DEPTH_TEST);
        self.context.depth_func(gl::LEQUAL);
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear_depth(1.0);

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
            .bind_buffer(gl::ELEMENT_ARRAY_BUFFER, Some(&self.cube.index_buffer));
        self.context
            .draw_elements(gl::TRIANGLES, 36, gl::UNSIGNED_SHORT, 0);

        window().request_animation_frame(move |time| {
            rc.borrow_mut().animate(time, rc.clone());
        });
    }
}

fn main() {
    stdweb::initialize();

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

    let cube = Cube.bind(&context);

    let shader = Shader::new(
        &context,
        r#"
        attribute vec3 position;
        uniform mat4 Pmatrix;
        uniform mat4 Vmatrix;
        uniform mat4 Mmatrix;
        attribute vec3 color;
        varying vec3 vColor;

        void main() {
            gl_Position = Pmatrix*Vmatrix*Mmatrix*vec4(position, 1.);
            vColor = color;
        }
    "#,
        r#"
        precision mediump float;
        varying vec3 vColor;

        void main() {
            gl_FragColor = vec4(vColor, 1.);
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

    context.bind_buffer(gl::ARRAY_BUFFER, Some(&cube.vertex_buffer));
    let position = context.get_attrib_location(&shader.program, "position") as u32;
    context.vertex_attrib_pointer(position, 3, gl::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(position);

    context.bind_buffer(gl::ARRAY_BUFFER, Some(&cube.color_buffer));
    let color = context.get_attrib_location(&shader.program, "color") as u32;
    context.vertex_attrib_pointer(color, 3, gl::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(color);

    context.use_program(Some(&shader.program));

    let state = Rc::new(RefCell::new(State {
        time_old: 0.0,
        mov_matrix: Matrix4::from_scale(1.),
        view_matrix: Matrix4::from_translation(vec3(0., 0., -6.)),
        canvas,
        context,
        p_matrix,
        v_matrix,
        m_matrix,
        cube,
    }));

    state.borrow_mut().animate(0., state.clone());

    stdweb::event_loop();
}
