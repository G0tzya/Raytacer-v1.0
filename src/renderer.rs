use core::f32;

use crate::primitives::primitives::*;
use crate::lights::*;
use rand::prelude::*;

use glam::Vec3;

pub struct Scene<'a> {
    pub objects: Vec<&'a dyn Primitives>,
    pub lights: Vec<&'a lights::Light>,
}

pub struct Camera {
    pub position: Vec3,
    pub look_at: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub background_color: Vec3,
}

pub fn render_function(
    x: usize,
    y: usize,
    width: i32,
    height: i32,
    camera: &Camera,
    scene: &Scene
) -> Vec3 {
    let u = (x as f32 / width as f32) * 2.0 - 1.0;
    let v = 1.0 - (y as f32 / height as f32) * 2.0;

    let rng = &mut rand::rng(); 

    let ray_origin = camera.position;
    let ray_direction = Vec3::new(u + rng.random::<f32>() * 0.003, v + rng.random::<f32>() * 0.003, -1.0).normalize();

    let mut closest_t = f32::INFINITY;
    let mut hit_normal = Vec3::ZERO;
    let mut hit_point = Vec3::ZERO;
    let mut hit_anything = false;

    let mut direct_hit_material: &Material = &Material::default();  

    let ray_bounce_depth= 6;

    // Find closest intersection
    for obj in scene.objects.iter() {
        if let Some((t, normal)) = obj.intersection(ray_origin, ray_direction) {
            if t < closest_t && t > 0.001 {
                closest_t = t;
                hit_normal = normal;
                hit_point = ray_origin + ray_direction * t;
                hit_anything = true;
                direct_hit_material = &obj.get_material()
            }
        }
    }

    if !hit_anything {
        return camera.background_color;
    }

    let perfect_reflect_direction = ray_direction.reflect(hit_normal);

    // Accumulate all lights
    let mut accumulated_light = Vec3::ZERO;

    for light in scene.lights.iter() {
        match light {
            lights::Light::Point(pl) => {
                let to_light = pl.position - hit_point;
                let distance = to_light.length();
                let light_dir = to_light / distance;

                // Shadow ray
                let shadow_origin = hit_point + hit_normal * 0.001;
                let mut in_shadow = false;

                for obj in scene.objects.iter() {
                    if let Some((t_shadow, _)) = obj.intersection(shadow_origin, light_dir) {
                        if t_shadow > 0.0001 && t_shadow < distance {
                            in_shadow = true;
                            break;
                        }
                    }
                }

                if in_shadow {continue}

                // Lambert diffuse
                let ndotl = hit_normal.dot(light_dir).max(0.0);
                let attenuation = pl.intensity / (distance * distance);
                accumulated_light += (direct_hit_material.color / std::f32::consts::PI) * (pl.color / 255.0 * attenuation) * ndotl // remove for weird cell shade
            }
        }
    }

    let random_hemisphere_vector= sample_cosine_hemisphere(hit_normal, rng)
        .lerp(perfect_reflect_direction, 1.0 - direct_hit_material.roughness);
    // let random_hemisphere_vector = ray_direction.reflect(hit_normal);

    let indirect_color = cast_ray(
        hit_point,
        random_hemisphere_vector,
        camera,
        scene,
        rng,
        0,
        ray_bounce_depth
    );

    // Convert object color to 0..1 range if it's 0..255
    let object_color = direct_hit_material.color / 255.0;

    // Multiply accumulated light by object color
    let mut shaded_color = object_color;

    shaded_color = shaded_color * (indirect_color + accumulated_light); // this fixed everything

    // Clamp final color to [0,1] per channel
    // shaded_color = shaded_color.min(Vec3::splat(1.0));

    // pack_color(shaded_color)
    shaded_color
}

fn cast_ray (
    ray_origin: Vec3, 
    ray_direction: Vec3, 
    camera: &Camera, 
    scene: &Scene, 
    random: &mut ThreadRng, 
    recursion_depth: i32,
    max_depth: i32
) -> Vec3 {
    if recursion_depth >= max_depth {
        return Vec3::ZERO
    }

    if random.random_range(0.0..1.0) > 0.90 {
        return Vec3::ZERO;
    }

    let mut closest_distance: f32 = f32::INFINITY; 
    let mut hit_normal: Vec3 = Vec3::ZERO;
    let mut hit_point: Vec3 = Vec3::ZERO;
    let mut hit_anything: bool = false;
    let mut light_dot: f32 = 0.0;

    let mut direct_hit_material: &Material = &Material::default();  

    // find first ray collision
    for obj in scene.objects.iter() {
        if let Some((t, normal)) = obj.intersection(ray_origin, ray_direction) {
            if t < closest_distance && t > 0.001 {
                closest_distance = t;
                hit_normal = normal;
                hit_point = ray_origin + ray_direction * t;
                hit_anything = true;
                direct_hit_material = obj.get_material();
            }
        }
    }

    // missed collision check
    if !hit_anything {
        return camera.background_color;
    }

    let perfect_reflect_direction = ray_direction.reflect(hit_normal);

    // calculate direct to collision point
    let mut direct_light = Vec3::ZERO;
    for light in scene.lights.iter() {
        match light {
            lights::Light::Point(point_light) => {
                let to_light = point_light.position - hit_point;
                let distance = to_light.length();
                let light_dir = to_light / distance;
                let shadow_origin = hit_point + hit_normal * 0.001;
                let mut in_shadow = false;

                for obj in scene.objects.iter() {
                    if let Some((t_shadow, _)) = obj.intersection(shadow_origin, light_dir) {
                        if t_shadow > 0.0001 && t_shadow < distance {
                            in_shadow = true;
                            break;
                        }
                    }
                }

                if in_shadow { continue; }

                light_dot = hit_normal.dot(light_dir).max(0.0);
                let attenuation = point_light.intensity / (distance * distance);
                // direct_light += point_light.color * ndotl * attenuation;
                direct_light += (direct_hit_material.color / std::f32::consts::PI) * (point_light.color / 255.0 * attenuation) * light_dot
            }
        }
    }

    // calculate indirect lighting recursivly n+1
    let random_hemisphere_vector= sample_cosine_hemisphere(hit_normal, random)
        .lerp(perfect_reflect_direction, 1.0 - direct_hit_material.roughness);
    // let random_hemisphere_vector = ray_direction.reflect(hit_normal);

    let indirect_light_value = cast_ray (
        hit_point + hit_normal * 0.001,
        random_hemisphere_vector,
        camera,
        scene,
        random,
        recursion_depth + 1,
        max_depth
    );

    let object_color = direct_hit_material.color / 255.0;

    let combined_color = object_color * (indirect_light_value + direct_light);

    return combined_color;
}

fn sample_cosine_hemisphere(n: Vec3, rng: &mut ThreadRng) -> Vec3 {
    // sample disk with sqrt transform
    let u1: f32 = rng.random();
    let u2: f32 = rng.random();

    let r = u1.sqrt();
    let theta = 2.0 * std::f32::consts::PI * u2;
    let x = r * theta.cos();
    let y = r * theta.sin();
    let z = (1.0 - u1).sqrt(); // ensures x^2 + y^2 + z^2 = 1

    // build tangent space
    let (t, b) = tangent_space(n);

    // world-space direction
    let dir = (t * x) + (b * y) + (n * z);
    dir.normalize()
}

fn tangent_space(n: Vec3) -> (Vec3, Vec3) {
    // choose helper vector to avoid degenerate cross
    let helper = if n.x.abs() > 0.1 { Vec3::Y } else { Vec3::X };
    let tangent = n.cross(helper).normalize();
    let bitangent = n.cross(tangent);
    (tangent, bitangent)
}

fn pack_color(c: Vec3) -> u32 {
    let gamma = 2.2;
    // gamma correction
    let corrected = Vec3::new(
        c.x.clamp(0.0, 1.0).powf(1.0 / gamma),
        c.y.clamp(0.0, 1.0).powf(1.0 / gamma),
        c.z.clamp(0.0, 1.0).powf(1.0 / gamma),
    );

    let r = (corrected.x * 255.0) as u8;
    let g = (corrected.y * 255.0) as u8;
    let b = (corrected.z * 255.0) as u8;
    let a = 255u8;

    ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
}
