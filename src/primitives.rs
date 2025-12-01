
pub mod primitives {
    use glam::Vec3;

    pub trait Primitives {
        fn intersection(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<(f32, Vec3)>;
        fn get_material(&self) -> &Material;
    }
    pub struct Sphere <'a> {
        pub center: Vec3,
        pub radius: f32,
        pub material: &'a Material
    }
    
    pub struct Plane <'a> {
        pub point: Vec3,
        pub normal: Vec3,
        pub material: &'a Material
    }

    #[derive(Default)]
    pub struct Material {
        pub color: Vec3,
        pub roughness: f32,
        pub emission: Vec3,
    }
    
    impl Primitives for Sphere <'_>{ // not sure why it needs an unspesified lifetime
        fn intersection(&self, ray_origin: Vec3, ray_dir: Vec3) -> Option<(f32, Vec3)> {
            let oc = ray_origin - self.center;
    
            let a = ray_dir.dot(ray_dir);                      
            let b = 2.0 * oc.dot(ray_dir);                      
            let c = oc.dot(oc) - self.radius * self.radius;      
    
            let discriminant = b*b - 4.0*a*c;
    
            if discriminant < 0.0 {
                return None;
            }
    
            // Quadratic solutions
            let sqrt_disc = discriminant.sqrt();
            let mut t1 = (-b - sqrt_disc) / (2.0 * a);
            let mut t2 = (-b + sqrt_disc) / (2.0 * a);
    
            // Sort so t1 is the nearest
            if t1 > t2 { std::mem::swap(&mut t1, &mut t2); }
    
            // Reject hits behind the ray
            let t = if t1 > 0.001 {       // small epsilon avoids self-intersection
                t1
            } else if t2 > 0.001 {
                t2
            } else {
                return None;
            };
    
            let hit_point = ray_origin + ray_dir * t;
            let normal = (hit_point - self.center).normalize();
    
            Some((t, normal))
        }

        fn get_material(&self) -> & Material {
            &self.material
        }
    }
    
    impl Primitives for Plane <'_>{
        fn intersection(&self, ray_origin: Vec3, ray_dir: Vec3) -> Option<(f32, Vec3)> {
            let denom = ray_dir.dot(self.normal);
    
            // Ray parallel to plane?
            if denom > 1e-6 {
                return None;
            }
    
            let t = (self.point - ray_origin).dot(self.normal) / denom;
    
            if t < 0.001 {
                return None; // behind ray or too close
            }
    
            Some((t, self.normal))
        }

        fn get_material(&self) -> &Material {
            &self.material
        }
    }
}
