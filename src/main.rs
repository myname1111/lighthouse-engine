#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::single_match)]
#![allow(unused_imports)]
#![allow(clippy::zero_ptr)]

const WINDOW_TITLE: &str = "Opengl tutorial";

use common_macros::*;
use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
};
use device_query::{DeviceQuery, DeviceState, Keycode, MouseState};
use image::DynamicImage;
use lighthouse::{
    core::{
        camera::{CameraSettings, CameraSettingsBuilder, CameraTrait},
        mouse::{MousePressed::*, StateOfMouse::*, *},
        object::{ControllableKey, ControllableMouse, Object, PosRot},
        world::{Enviroment, World},
    },
    graphics::{buffer::*, shader::*, texture::*, uniform::*, vertex::*, *},
};
use nalgebra_glm::*;
use std::thread::sleep;
use std::time::*;
use std::{borrow::BorrowMut, fs};

type Vertex = [f32; 5];
type TriIndexes = [u32; 3];

const VERTICES: [Vertex; 5] = [
    [0.5, -0.5, 0.0, 1.0, 0.0],   // front right
    [-0.5, -0.5, 0.0, 0.0, 0.0],  // front left
    [-0.5, -0.5, -1.0, 1.0, 0.0], // back left
    [0.5, -0.5, -1.0, 0.0, 0.0],  // back right
    [0.0, 0.5, -0.5, 0.5, 1.5],   // top
];

const INDICES: [TriIndexes; 4] = [[0, 1, 4], [1, 2, 4], [2, 3, 4], [0, 3, 4]];

const WIDTH: u16 = 800;
const HEIGHT: u16 = 600;

struct Camera<'a> {
    pos: Vec3,
    rot: Vec3,
    settings: CameraSettings<'a>,
}

impl<'a> Camera<'a> {
    pub fn new(pos: Vec3, rot: Vec3, settings: CameraSettings<'a>, world: World<'a>) -> Self {
        Camera::<'a> { pos, rot, settings }
    }
}

impl<'a> PosRot for Camera<'a> {
    fn get_pos(&self) -> &Vec3 {
        &self.pos
    }

    fn get_rot(&self) -> &Vec3 {
        &self.rot
    }

    fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    fn set_rot(&mut self, rot: Vec3) {
        self.rot = rot;
    }
}

impl<'a> Object for Camera<'a> {
    fn update(&mut self, world: &World) {
        todo!()
    }
}

impl<'a> CameraTrait for Camera<'a> {
    fn get_camera_settings(&self) -> CameraSettings {
        self.settings
    }
}

impl<'a> ControllableKey for Camera<'a> {
    fn on_key(&mut self, keys: Vec<Keycode>) {
        for key in keys {
            match key {
                Keycode::W => self.pos.z += 0.01,
                Keycode::A => self.pos.x += 0.01,
                Keycode::S => self.pos.z -= 0.01,
                Keycode::D => self.pos.x -= 0.01,
                Keycode::LShift | Keycode::RShift => self.pos.y -= 0.01,
                Keycode::Space => self.pos.y += 0.01,
                _ => (),
            }
        }
    }
}

impl<'a> ControllableMouse for Camera<'a> {
    fn on_mouse(&mut self, mouse: &mut Mouse, device: &mut DeviceState) {
        if let Some(keys) = mouse.get_pressed_cooldown(Duration::from_millis(100)) {
            keys.iter().for_each(|key| match key {
                LeftMouse => mouse.state = Locked(self.settings.screen_size / 2.0),
                RightMouse => mouse.state = Free,
                _ => (),
            });
        }
        // for pressed in mouse.get_pressed_cooldown(Duration::from_millis(100)) {
        //     match pressed {
        //         LeftMouse => mouse.state = Locked(self.settings.screen_size / 2.0),
        //         RightMouse => mouse.state = Free,
        //         _ => (),
        //     }
        // }

        match mouse.state {
            Free => (),
            Locked(vec) => {
                let arr: [f32; 2] = vec.into();
                let (x, y) = (arr[0], arr[1]);

                self.settings.win.warp_mouse_in_window(x as i32, y as i32);
                *device = DeviceState::new();
                mouse.mouse = device.get_mouse();
            }
        }
    }
}

fn main() {
    let vert = fs::read_to_string("shaders/vert.glsl").expect("Failed to read vertex shader");
    let frag = fs::read_to_string("shaders/frag.glsl").expect("Failed to read fragment shader");

    let vert = vert.as_str();
    let frag = frag.as_str();

    // Create a new device state
    let mut device_state = DeviceState::new();
    let mut mouse: Mouse = device_state.clone().into();

    let sdl = SDL::init(InitFlags::Everything).expect("couldn't start SDL");
    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core)
        .unwrap();
    #[cfg(target_os = "macos")]
    {
        sdl.gl_set_attribute(SdlGlAttr::Flags, ContextFlag::ForwardCompatible)
            .unwrap();
    }
    let win = sdl
        .create_gl_window(
            WINDOW_TITLE,
            WindowPosition::Centered,
            WIDTH.into(),
            HEIGHT.into(),
            WindowFlags::Shown,
        )
        .expect("couldn't make a window and context");
    win.set_swap_interval(SwapInterval::Vsync);

    unsafe {
        load_gl_with(|f_name| win.get_proc_address(f_name));
    }

    clear_color(0.2, 0.3, 0.3, 1.0); // sets background color

    let vao = VertexArray::new().expect("Couldn't make a VAO");
    vao.bind();

    let vbo = Buffer::new().expect("Couldn't make a VBO");
    vbo.bind(BufferType::Array);
    buffer_data(
        BufferType::Array,
        bytemuck::cast_slice(&VERTICES),
        GL_STATIC_DRAW,
    );

    let ebo = Buffer::new().expect("Couldn't make EBO");
    ebo.bind(BufferType::ElementArray);
    buffer_data(
        BufferType::ElementArray,
        bytemuck::cast_slice(&INDICES),
        GL_STATIC_DRAW,
    );

    unsafe {
        glVertexAttribPointer(
            0,
            3,
            GL_FLOAT,
            GL_FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            0 as *const _,
        );
        glVertexAttribPointer(
            1,
            2,
            GL_FLOAT,
            GL_FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            size_of::<[f32; 3]>() as *const _,
        );

        glEnableVertexAttribArray(0);
        glEnableVertexAttribArray(1);
    }
    let shader_program = ShaderProgram::from_vert_frag(vert, frag).unwrap();
    shader_program.use_program();

    // World
    let world = World::new(
        Enviroment::new(
            vec2(WIDTH.into(), HEIGHT.into()),
            &win,
            &shader_program,
            &device_state,
            &mouse,
        ),
        &mut Camera::new(
            vec3(0.0, 0.0, -2.0),
            vec3(0.0, 0.0, 1.0),
            CameraSettingsBuilder::default()
                .screen_size(vec2(WIDTH.into(), HEIGHT.into()))
                .win(&win)
                .shader_program(&shader_program)
                .build(),
            world,
        ),
        Vec::new(),
    );

    // textures
    let img = image::io::Reader::open("data/image.jpg")
        .unwrap()
        .decode()
        .unwrap();
    let mut texture = Texture::from_image(
    GL_TEXTURE0,
    GL_TEXTURE_2D,
    hash_map!{
      "GL_TEXTURE_MIN_FILTER" => number::MultiSingularNumber::Number(number::Number::Integer(GL_NEAREST as i32)),
      "GL_TEXTURE_MAG_FILTER" => number::MultiSingularNumber::Number(number::Number::Integer(GL_LINEAR as i32)),
      "GL_TEXTURE_WRAP_S" => number::MultiSingularNumber::Number(number::Number::Integer(GL_REPEAT as i32)),
      "GL_TEXTURE_WRAP_T" => number::MultiSingularNumber::Number(number::Number::Integer(GL_REPEAT as i32))
    },
    0,
    img
  ).unwrap();

    // uniforms
    Uniform::new(&shader_program, "tex_color");

    // enable depth buffer
    enable(GL_DEPTH_TEST);
    camera.matrix("camera_matrix");
    // Location of the world
    'main_loop: loop {
        mouse.mouse = device_state.get_mouse();

        // handle events this frame
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => break 'main_loop,
                _ => (),
            }
        }

        camera.on_key(device_state.get_keys());
        camera.on_mouse(&mut mouse, &mut device_state);
        camera.matrix("camera_matrix");

        texture.bind(GL_TEXTURE_2D);

        // and then draw!
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT);
            glClear(GL_DEPTH_BUFFER_BIT);
            glDrawElements(
                GL_TRIANGLES,
                (size_of::<TriIndexes>() * INDICES.len())
                    .try_into()
                    .unwrap(),
                GL_UNSIGNED_INT,
                0 as *const _,
            );
        }
        win.swap_window();
    }
}
