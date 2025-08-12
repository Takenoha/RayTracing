type Csmesh = csgrs::mesh::Mesh<()>;
use raytracing_core::Hittable;
pub trait Hittable_beby: Hittable {
    fn to_mesh(&self) -> Option<Csmesh>;
}
