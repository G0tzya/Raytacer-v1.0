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

    if random.random_range(0.0..1.0) > 0.84 {
        return Vec3::ZERO;
    }

    let mut closest_distance: f32 = f32::INFINITY; 
    let mut hit_normal: Vec3 = Vec3::ZERO;
    let mut hit_point: Vec3 = Vec3::ZERO;
    let mut hit_anything: bool = false;
    let mut direct_hit_color: Vec3 = Vec3::ZERO;
    let mut light_dot: f32 = 0.0;

    // find first ray collision
    for obj in scene.objects.iter() {
        if let Some((t, normal)) = obj.intersection(ray_origin, ray_direction) {
            if t < closest_distance && t > 0.001 {
                closest_distance = t;
                hit_normal = normal;
                hit_point = ray_origin + ray_direction * t;
                hit_anything = true;
                direct_hit_color = obj.get_color();
            }
        }
    }

    // missed collision check
    if !hit_anything {
        return camera.background_color;
    }

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
                direct_light += (direct_hit_color / std::f32::consts::PI) * (point_light.color / 255.0 * attenuation) * light_dot
            }
        }
    }

    // calculate indirect lighting recursivly
    let random_hemisphere_vector= sample_cosine_hemisphere(hit_normal, random);

    let indirect_light_value = cast_ray (
        hit_point + hit_normal * 0.001,
        random_hemisphere_vector,
        camera,
        scene,
        random,
        recursion_depth + 1,
        max_depth
    );

    let object_color = direct_hit_color / 255.0;

    let combined_color = object_color * (indirect_light_value + direct_light);

    return combined_color;
}