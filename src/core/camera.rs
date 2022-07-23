use std::time::Duration;

use super::{
    mouse::{Mouse, StateOfMouse::*},
    object::*,
};
use crate::{
    core::mouse::MousePressed,
    graphics::{shader::ShaderProgram, uniform::Uniform},
};
use beryllium::GlWindow;
use device_query::{DeviceQuery, DeviceState, Keycode};
use nalgebra_glm::*;

/// Builder for [CameraSettings]
///
/// # Example
/// ```
/// // here are the required dependencies
/// let settings = CameraSettingsBuilder::new()
///     .screen_size(size)
///     .win(&win)
///     .shader_program(&shader_program)
///     // Here are the optional ones, they are filled with these default values
///     .fov(45.0)
///     .sensitivity(1.0)
///     .near_plane(0.1)
///     .far_plane(100.0)
///     .build() // And finally build
/// ```
#[derive(Copy, Clone)]
pub struct CameraSettingsBuilder<'a> {
    /// This field is supposed to store the width of the screen
    screen_size: Option<Vec2>,
    /// FOV of the camera(in degrees)
    fov: f32,
    /// Sensitivity of the mouse
    sensitivity: f32,
    /// Window
    win: Option<&'a GlWindow>,
    /// Anything below this value will be clipped
    near_plane: f32,
    /// Anything above this value will be clipped
    far_plane: f32,
    /// The shader program
    shader_program: Option<&'a ShaderProgram>,
}

impl<'a> CameraSettingsBuilder<'a> {
    /// Creates a new camera settings
    pub fn new() -> Self {
        CameraSettingsBuilder::<'a> {
            screen_size: None,
            fov: 45.0,
            win: None,
            sensitivity: 1.0,
            near_plane: 0.1,
            far_plane: 100.0,
            shader_program: None,
        }
    }

    /// This function is supposed to set the screen_size. It must be called
    pub fn screen_size(&mut self, screen_size: Vec2) -> &mut Self {
        self.screen_size = Some(screen_size);
        self
    }

    /// This function is supposed to set the fov. It is optional
    pub fn fov(&mut self, fov: f32) -> &mut Self {
        self.fov = fov;
        self
    }

    /// This function is supposed to set the sensitivity of the mouse. It is optional
    pub fn sensitivity(&mut self, sensitivity: f32) -> &mut Self {
        self.sensitivity = sensitivity;
        self
    }

    /// This function is supposed to set the win. It must be called
    pub fn win(&mut self, win: &'a GlWindow) -> &mut Self {
        self.win = Some(win);
        self
    }

    /// This function is supposed to set the near_plane. It is optional
    pub fn near_plane(&mut self, near_plane: f32) -> &mut Self {
        self.near_plane = near_plane;
        self
    }

    /// This function is supposed to set the far_plane. It is optional
    pub fn far_plane(&mut self, far_plane: f32) -> &mut Self {
        self.far_plane = far_plane;
        self
    }

    /// This function is supposed to set the shader_program. It must be called
    pub fn shader_program(&mut self, shader_program: &'a ShaderProgram) -> &mut Self {
        self.shader_program = Some(shader_program);
        self
    }

    /// Build the settings for the camera
    ///
    /// NOTE: will panic if an argument isn't default or specified
    pub fn build(&self) -> CameraSettings<'a> {
        CameraSettings::<'a> {
            screen_size: self.screen_size.expect("Error: argument screen width is not satisfied\nhelp: you can call .screen_width"),
            fov: 45.0,
            sensitivity: self.sensitivity,
            win: self.win.expect("Error: argument window is not satisfied\nhelp: you can call .win"),
            near_plane: 0.1,
            far_plane: 100.0,
            shader_program: self.shader_program.expect("Error: argument shadeer program is not satisfied\nhelp: you can call .shader_program"),
        }
    }
}

impl<'a> Default for CameraSettingsBuilder<'a> {
    /// Creates a new camera settings
    fn default() -> Self {
        Self::new()
    }
}

/// Setting for the [Camera] struct
///
/// # Examples
/// Make a new setting using [CameraSettingsBuilder]
/// ```
/// let camera_settings = CameraSettingsBuilder::new().
///     win(&win)
///     ... // see CameraSettingsBuilder
/// ```
/// load it into [Camera]
/// ```
/// let camera = Camera::new(pos, rot, settings);
/// ```
#[derive(Copy, Clone)]
pub struct CameraSettings<'a> {
    /// This field is supposed to store the width of the screen
    pub screen_size: Vec2,
    /// FOV of the camera(in degrees)
    pub fov: f32,
    /// Sensitivity of the mouse
    pub sensitivity: f32,
    /// Window
    pub win: &'a GlWindow,
    /// anything below this value will be clipped
    pub near_plane: f32,
    /// anything above this value will be clipped
    pub far_plane: f32,
    /// the shader program
    pub shader_program: &'a ShaderProgram,
}

/// Camera trait responsible for the DefaultCamera struct. TODO: move DefaultCamera into Camera, ContorllabeMouse ... and users can implement
///
/// You dont have to implement matrix. You do however need to implement get_camera_settings for the
/// default implementation to work
/// # Examples
/// Make a new Camera
/// ```
/// impl Camera for MyCamera {
///     fn get_camera_settings() {
///         self.settings
///     }
/// }
/// ```
pub trait Camera: Object {
    /// Creates a new matrix from the camera position and parameters
    fn matrix(&self, uniform: &'static str) {
        let settings = self.get_camera_settings();

        let identity = mat4(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );

        let model = identity;

        let view = look_at(
            &self.get_pos(),
            &(self.get_pos() + self.get_rot()),
            &vec3(0.0, 1.0, 0.0),
        );
        let proj = perspective::<f32>(
            settings.screen_size.x / settings.screen_size.y,
            settings.fov.to_radians(),
            settings.near_plane,
            settings.far_plane,
        );

        Uniform::new(self.get_camera_settings().shader_program, uniform)
            .set_uniform_matrix(false, (proj * view * model).into())
    }

    /// Get the camera settings
    fn get_camera_settings(&self) -> CameraSettings;
}

/// Defalut Camera struct with default implementation
pub struct DefaultCamera<'a> {
    /// This field is supposed to store positional information
    pub pos: Vec3,
    /// This field is supposed to store rotational information
    pub rot: Vec3,
    /// settings for the camera
    pub settings: CameraSettings<'a>,
}

impl<'a> DefaultCamera<'a> {
    /// Creates a new camera
    ///
    /// # Arguments
    ///
    /// pos: Vec3 is supposed to store positional information
    /// rot: Vec3 is supposed to store rotational information
    /// width: i32 is supposed to store the width of the camera
    /// height: i32 is supposed to store the height of the camera
    /// speed_pos: Vec3 is supposed to store the rotational speed of the camera
    /// speed_rot: Vec3 is supposed to store the rotational speed of the camera
    /// sensitivity: f32 is supposed to store the height of the camera
    pub fn new(pos: Vec3, rot: Vec3, settings: CameraSettings<'a>) -> Self {
        DefaultCamera::<'a> { pos, rot, settings }
    }
}

impl<'a> Object for DefaultCamera<'a> {
    fn update(&mut self) {}

    fn get_pos(&self) -> Vec3 {
        self.pos
    }

    fn get_rot(&self) -> Vec3 {
        self.rot
    }

    fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    fn set_rot(&mut self, rot: Vec3) {
        self.rot = rot;
    }
}

impl<'a> Camera for DefaultCamera<'a> {
    fn get_camera_settings(&self) -> CameraSettings {
        self.settings
    }
}

impl<'a> ControllableKey for DefaultCamera<'a> {
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

impl<'a> ControllableMouse for DefaultCamera<'a> {
    fn on_mouse(&mut self, mouse: &mut Mouse, device: &mut DeviceState) {
        for pressed in mouse.get_pressed_cooldown(Duration::from_millis(100)) {
            match pressed {
                MousePressed::LeftMouse => mouse.state = Locked(self.settings.screen_size / 2.0),
                MousePressed::RightMouse => mouse.state = Free,
                _ => (),
            }
        }

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
