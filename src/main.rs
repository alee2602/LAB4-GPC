
use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{look_at, perspective, Mat4, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

mod camera;
mod color;
mod fragment;
mod framebuffer;
mod obj;
mod shaders;
mod triangle;
mod vertex;

use camera::Camera;
use fastnoise_lite::{FastNoiseLite, NoiseType};
use framebuffer::Framebuffer;
use obj::Obj;
use shaders::{fragment_shader, vertex_shader, ShaderType};
use triangle::triangle;
use vertex::Vertex;

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: FastNoiseLite,
}

fn create_noise() -> FastNoiseLite {
    create_cloud_noise()
}

fn create_cloud_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, cos_x, -sin_x, 0.0, 0.0, sin_x, cos_x, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y, 0.0, sin_y, 0.0, 0.0, 1.0, 0.0, 0.0, -sin_y, 0.0, cos_y, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0, sin_z, cos_z, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale,
        0.0,
        0.0,
        translation.x,
        0.0,
        scale,
        0.0,
        translation.y,
        0.0,
        0.0,
        scale,
        translation.z,
        0.0,
        0.0,
        0.0,
        1.0,
    );

    transform_matrix * rotation_matrix
}

fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0,
        0.0,
        0.0,
        width / 2.0,
        0.0,
        -height / 2.0,
        0.0,
        height / 2.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    current_shader: &ShaderType,
) {
    // Vertex Shader
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Fragment Processing
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;

        if x < framebuffer.width && y < framebuffer.height {
            let shaded_color = fragment_shader(&fragment, &uniforms, current_shader);
            let color = shaded_color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Animated Fragment Shader",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x333355);

    // model position
    let translation = Vec3::new(0.0, 0.0, 0.0);
    let rotation = Vec3::new(0.0, 0.0, 0.0);
    let scale = 1.0f32;

    // camera parameters
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let obj = Obj::load("assets/models/sphere.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array();
    let moon_obj = Obj::load("assets/models/moon.obj").expect("Failed to load moon obj");
    let moon_vertex_array = moon_obj.get_vertex_array();

    let mut current_shader = ShaderType::RockyPlanet;
    let mut time = 0;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        if window.is_key_down(Key::Key1) {
            current_shader = ShaderType::GasGiant;
        }
        if window.is_key_down(Key::Key2) {
            current_shader = ShaderType::ColdGasGiant;
        }
        if window.is_key_down(Key::Key3) {
            current_shader = ShaderType::Solar;
        }
        if window.is_key_down(Key::Key4) {
            current_shader = ShaderType::RockyPlanet;
        }
        if window.is_key_down(Key::Key5) {
            current_shader = ShaderType::RockyPlanetVariant;
        }
        if window.is_key_down(Key::Key6) {
            current_shader = ShaderType::AlienPlanet;
        }
        if window.is_key_down(Key::Key7) {
            current_shader = ShaderType::GlacialTextured;
        }

        time += 1;

        handle_input(&window, &mut camera);

        framebuffer.clear();

        let noise = create_noise();
        let model_matrix = create_model_matrix(translation, scale, rotation);
        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        let projection_matrix =
            create_perspective_matrix(window_width as f32, window_height as f32);
        let viewport_matrix =
            create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
        let uniforms = Uniforms {
            model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            noise,
        };

        framebuffer.set_current_color(0xFFDDDD);
        render(&mut framebuffer, &uniforms, &vertex_arrays, &current_shader);

        if current_shader == ShaderType::RockyPlanet {
            let orbit_radius = 2.0; // Radio de la órbita de la luna alrededor del planeta
            let orbit_speed = 0.005; // Velocidad de la órbita de la luna
            let moon_x = orbit_radius * (time as f32 * orbit_speed).cos();
            let moon_z = orbit_radius * (time as f32 * orbit_speed).sin();

            let moon_translation = Vec3::new(moon_x, 0.0, moon_z);
            let moon_scale = 0.15; 
            let moon_model_matrix =
                create_model_matrix(moon_translation, moon_scale, Vec3::new(0.0, 0.0, 0.0));

            let moon_noise = create_noise(); 

            let moon_uniforms = Uniforms {
                model_matrix: moon_model_matrix,
                view_matrix: uniforms.view_matrix,
                projection_matrix: uniforms.projection_matrix,
                viewport_matrix: uniforms.viewport_matrix,
                time: uniforms.time,
                noise: moon_noise, 
            };
            render(
                &mut framebuffer,
                &moon_uniforms,
                &moon_vertex_array,
                &ShaderType::Moon,
                
            );
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

fn handle_input(window: &Window, camera: &mut Camera) {
    let movement_speed = 1.0;
    let rotation_speed = PI / 50.0;
    let zoom_speed = 0.1;

    //  camera orbit controls
    if window.is_key_down(Key::Left) {
        camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
        camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::W) {
        camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::S) {
        camera.orbit(0.0, rotation_speed);
    }

    // Camera movement controls
    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
        movement.x -= movement_speed;
    }
    if window.is_key_down(Key::D) {
        movement.x += movement_speed;
    }
    if window.is_key_down(Key::Q) {
        movement.y += movement_speed;
    }
    if window.is_key_down(Key::E) {
        movement.y -= movement_speed;
    }
    if movement.magnitude() > 0.0 {
        camera.move_center(movement);
    }

    // Camera zoom controls
    if window.is_key_down(Key::Up) {
        camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.zoom(-zoom_speed);
    }
}
