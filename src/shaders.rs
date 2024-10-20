use crate::color::Color;
use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::Uniforms;
use nalgebra_glm::{dot, mat4_to_mat3, Mat3, Vec3, Vec4};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::f32::consts::PI;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);

    let transformed =
        uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    let w = transformed.w;
    let transformed_position =
        Vec4::new(transformed.x / w, transformed.y / w, transformed.z / w, 1.0);

    let screen_position = uniforms.viewport_matrix * transformed_position;

    let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix = model_mat3
        .transpose()
        .try_inverse()
        .unwrap_or(Mat3::identity());

    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transformed_normal,
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    //gas_giant_shader(fragment, uniforms)
    cold_gas_giant_shader(fragment, uniforms)
}

pub fn gas_giant_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let base_colors = [
        Vec3::new(110.0 / 255.0, 0.0 / 255.0, 90.0 / 255.0),    
        Vec3::new(160.0 / 255.0, 20.0 / 255.0, 60.0 / 255.0),   
        Vec3::new(130.0 / 255.0, 10.0 / 255.0, 80.0 / 255.0),   
        Vec3::new(180.0 / 255.0, 40.0 / 255.0, 90.0 / 255.0),   
        Vec3::new(140.0 / 255.0, 10.0 / 255.0, 70.0 / 255.0),   
    ];

    let time = uniforms.time as f32 * 0.001; 
    let dynamic_y = fragment.vertex_position.y + time;

    let distortion_scale = 10.0; 
    let distortion_value = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * distortion_scale,
        dynamic_y * distortion_scale,
    );

    // Se modifica la posición 'y' con la distorsión para crear bandas más suaves y añadir variación en 'x'
    let distorted_y = dynamic_y + distortion_value * 0.1 + fragment.vertex_position.x * 0.05;

    let band_frequency = 40.0;
    let band_sine = (distorted_y * band_frequency).sin();
    let band_variation = (fragment.vertex_position.y * 10.0).sin() * 0.3; 
    let band_index_float = (band_sine + band_variation + 1.0) / 2.0 * (base_colors.len() as f32);
    let band_index = band_index_float as usize % base_colors.len();
    let mut rng = rand::thread_rng();
    let random_offset: f32 = rng.gen_range(-0.03..0.03); 
    let base_band_color =
        base_colors[band_index] + Vec3::new(random_offset, random_offset, random_offset);

    // Aumentar la saturación de algunas bandas de forma aleatoria
    let saturation_boost: f32 = if rng.gen_bool(0.5) { 1.2 } else { 1.0 };
    let boosted_band_color = base_band_color * saturation_boost;

    // Se elige el siguiente color de banda para suavizar la transición
    let next_band_index = (band_index + 1) % base_colors.len();
    let next_band_color =
        base_colors[next_band_index] + Vec3::new(random_offset, random_offset, random_offset);

    // Interpolación suave entre colores adyacentes
    let interpolation_factor = band_index_float.fract();
    let interpolated_color = boosted_band_color.lerp(&next_band_color, interpolation_factor);

    // capas de ruido de alta frecuencia para dar más textura a las bandas
    let noise_scale_1 = 80.0; 
    let noise_value_1 = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * noise_scale_1,
        fragment.vertex_position.y * noise_scale_1,
    );

    let noise_scale_2 = 40.0;
    let noise_value_2 = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * noise_scale_2,
        fragment.vertex_position.y * noise_scale_2,
    );

    let perturbed_color = interpolated_color * (0.95 + (noise_value_1 + noise_value_2) * 0.015); 

    let internal_shadow = (distorted_y * band_frequency * 0.1).sin().abs() * 0.15; 
    let shaded_color = perturbed_color * (1.0 - internal_shadow);

    let shadow_noise_scale = 50.0;
    let shadow_noise = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * shadow_noise_scale,
        fragment.vertex_position.y * shadow_noise_scale,
    );
    let shadow_variation = 1.0 - shadow_noise * 0.05; 
    let final_shaded_color = shaded_color * shadow_variation;
    let spot_noise_scale = 25.0;
    let spot_noise = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * spot_noise_scale,
        fragment.vertex_position.y * spot_noise_scale,
    );

    let mut final_color;

    if spot_noise > 0.75 {
        let mix_factor = (spot_noise - 0.75) / 0.25;
        let storm_color = Vec3::new(0.95, 0.85, 0.65);
        final_color = final_shaded_color.lerp(&storm_color, mix_factor);
    } else {
        final_color = final_shaded_color;
    }

    let normal = fragment.vertex_position.normalize();

    let light_dir = Vec3::new(0.6, 0.8, 0.4).normalize();
    let lambertian = light_dir.dot(&normal).max(0.0);
    let shading_factor = 0.75 + 0.25 * lambertian;

    final_color = final_color * shading_factor;

    // dispersión atmosférica
    let gradient_shading = 1.0 - (fragment.vertex_position.y.abs() * 0.15); 
    final_color = final_color * gradient_shading;

    // reflejos especulares para simular brillos en la atmósfera
    let view_dir = Vec3::new(0.0, 0.0, 1.0).normalize();
    let reflect_dir = (2.0 * normal.dot(&light_dir) * normal - light_dir).normalize();
    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(10.0); 

    final_color = final_color + Vec3::new(1.0, 1.0, 1.0) * specular_intensity * 0.15;

    final_color = final_color * fragment.intensity;

    Color::new(
        (final_color.x * 255.0) as u8,
        (final_color.y * 255.0) as u8,
        (final_color.z * 255.0) as u8,
    )
}

pub fn cold_gas_giant_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let base_colors = [
        Vec3::new(100.0 / 255.0, 150.0 / 255.0, 180.0 / 255.0), 
        Vec3::new(120.0 / 255.0, 180.0 / 255.0, 200.0 / 255.0), 
        Vec3::new(90.0 / 255.0, 140.0 / 255.0, 170.0 / 255.0),  
        Vec3::new(130.0 / 255.0, 190.0 / 255.0, 210.0 / 255.0), 
        Vec3::new(80.0 / 255.0, 120.0 / 255.0, 160.0 / 255.0),  
    ];

    let time = uniforms.time as f32 * 0.001;
    let dynamic_y = fragment.vertex_position.y + time;

    let distortion_scale = 10.0;
    let distortion_value = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * distortion_scale,
        dynamic_y * distortion_scale,
    );

    let wind_tilt = fragment.vertex_position.x * 0.02;
    let distorted_y = dynamic_y + wind_tilt + distortion_value * 0.1 + fragment.vertex_position.x * 0.05;

    let band_frequency = 40.0;
    let band_sine = (distorted_y * band_frequency).sin();
    let band_variation = (fragment.vertex_position.y * 10.0).sin() * 0.3;
    let band_index_float = (band_sine + band_variation + 1.0) / 2.0 * (base_colors.len() as f32);
    let band_index = band_index_float as usize % base_colors.len();
    let mut rng = rand::thread_rng();
    let random_offset: f32 = rng.gen_range(-0.03..0.03);
    let base_band_color =
        base_colors[band_index] + Vec3::new(random_offset, random_offset, random_offset);

    let saturation_boost: f32 = if rng.gen_bool(0.5) { 1.2 } else { 1.0 };
    let boosted_band_color = base_band_color * saturation_boost;

    let next_band_index = (band_index + 1) % base_colors.len();
    let next_band_color =
        base_colors[next_band_index] + Vec3::new(random_offset, random_offset, random_offset);

    let interpolation_factor = band_index_float.fract();
    let interpolated_color = boosted_band_color.lerp(&next_band_color, interpolation_factor);

    let noise_scale_1 = 80.0;
    let noise_value_1 = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * noise_scale_1,
        fragment.vertex_position.y * noise_scale_1,
    );

    let noise_scale_2 = 40.0;
    let noise_value_2 = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * noise_scale_2,
        fragment.vertex_position.y * noise_scale_2,
    );

    let perturbed_color = interpolated_color * (0.95 + (noise_value_1 + noise_value_2) * 0.015);

    let internal_shadow = (distorted_y * band_frequency * 0.1).sin().abs() * 0.15;
    let shaded_color = perturbed_color * (1.0 - internal_shadow);

    let shadow_noise_scale = 50.0;
    let shadow_noise = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * shadow_noise_scale,
        fragment.vertex_position.y * shadow_noise_scale,
    );
    let shadow_variation = 1.0 - shadow_noise * 0.05;
    let final_shaded_color = shaded_color * shadow_variation;

    let spot_noise_scale = 15.0; 
    let spot_noise = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * spot_noise_scale,
        fragment.vertex_position.y * spot_noise_scale,
    );

    let mut final_color;

    if spot_noise > 0.7 {
        let mix_factor = (spot_noise - 0.7) / 0.3;
        let storm_color = Vec3::new(0.75, 0.85, 0.95); 
        final_color = final_shaded_color.lerp(&storm_color, mix_factor);
    } else {
        final_color = final_shaded_color;
    }

    let normal = fragment.vertex_position.normalize();

    let light_dir = Vec3::new(0.6, 0.8, 0.4).normalize();
    let lambertian = light_dir.dot(&normal).max(0.0);
    let shading_factor = 0.75 + 0.25 * lambertian;
    final_color = final_color * shading_factor;

    let gradient_shading = 1.0 - (fragment.vertex_position.y.abs() * 0.15);
    final_color = final_color * gradient_shading;

    let view_dir = Vec3::new(0.0, 0.0, 1.0).normalize();
    let reflect_dir = (2.0 * normal.dot(&light_dir) * normal - light_dir).normalize();
    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(10.0);
    final_color = final_color + Vec3::new(1.0, 1.0, 1.0) * specular_intensity * 0.15;

    final_color = final_color * fragment.intensity;

    Color::new(
        (final_color.x * 255.0) as u8,
        (final_color.y * 255.0) as u8,
        (final_color.z * 255.0) as u8,
    )
}