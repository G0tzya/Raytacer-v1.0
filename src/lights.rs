pub mod lights {
    use glam::Vec3;

    pub struct PointLight {
        pub position: Vec3,
        pub intensity: f32,
        pub color: Vec3,
    }

    pub enum Light {
        Point(PointLight),
        // Future:
        // Directional(DirectionalLight),
        // Spot(SpotLight),
    }
}
