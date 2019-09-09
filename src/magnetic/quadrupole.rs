extern crate nalgebra;
extern crate specs;
use crate::atom::Position;

use crate::magnetic::MagneticFieldSampler;
use nalgebra::Vector3;
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};
/// A component representing a 3D quadrupole field.
pub struct QuadrupoleField3D {
    /// Gradient of the quadrupole field, in units of Tesla/m
    gradient: f64,
    /// A unit vector pointing along the symmetry axis of the 3D quadrupole field.
    direction: Vector3<f64>,
}
impl QuadrupoleField3D {
    /// Creates a `QuadrupoleField3D` component with gradient specified in Gauss per cm.
    pub fn gauss_per_cm(gradient: f64, direction: Vector3<f64>) -> Self {
        Self {
            gradient: gradient * 0.01,
            direction: direction.normalize(),
        }
    }
}

impl Component for QuadrupoleField3D {
    type Storage = HashMapStorage<Self>;
}

/// Updates the values of magnetic field samplers to include quadrupole fields in the world.
pub struct Sample3DQuadrupoleFieldSystem;

impl Sample3DQuadrupoleFieldSystem {
    /// Calculates the quadrupole magnetic field.
    /// The field is defined with components `Bx = grad*x`, `By = grad*y`, `Bz = -2 * grad * z`.
    ///
    /// # Arguments
    ///
    /// `pos`: position of the sampler, m
    ///
    /// `centre`: position of the quadrupole node, m
    ///
    /// `gradient`: quadrupole gradient, in Tesla/m
    ///
    /// `direction`: A _normalized_ vector pointing in the direction of the quadrupole's symmetry axis.
    pub fn calculate_field(
        pos: Vector3<f64>,
        centre: Vector3<f64>,
        gradient: f64,
        direction: Vector3<f64>,
    ) -> Vector3<f64> {
        let delta = pos - centre;
        let z_comp = delta.dot(&direction) * direction;
        let r_comp = delta - z_comp;
        gradient * (r_comp - 2.0 * z_comp)
    }
}

impl<'a> System<'a> for Sample3DQuadrupoleFieldSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, QuadrupoleField3D>,
    );
    fn run(&mut self, (mut sampler, pos, quadrupole): Self::SystemData) {
        for (centre, quadrupole) in (&pos, &quadrupole).join() {
            for (pos, mut sampler) in (&pos, &mut sampler).join() {
                let quad_field = Sample3DQuadrupoleFieldSystem::calculate_field(
                    pos.pos,
                    centre.pos,
                    quadrupole.gradient,
                    quadrupole.direction,
                );
                sampler.field = sampler.field + quad_field;
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    extern crate nalgebra;
    use nalgebra::Vector3;

    /// Tests the correct implementation of the quadrupole 3D field
    #[test]
    fn test_quadrupole_3d_field() {
        let pos = Vector3::new(1.0, 1.0, 1.0);
        let centre = Vector3::new(0., 1., 0.);
        let gradient = 1.;
        let field =
            Sample3DQuadrupoleFieldSystem::calculate_field(pos, centre, gradient, Vector3::z());
        assert_eq!(field, Vector3::new(1., 0., -2.));
    }
}
