//! Loading a Sr 3D MOT directly from an oven source.

extern crate magneto_optical_trap as lib;
extern crate nalgebra;
use lib::atom::{AtomInfo, Position, Velocity};
use lib::atom_sources::emit::AtomNumberToEmit;
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::atom_sources::oven::{Oven, OvenAperture};
use lib::destructor::ToBeDestroyed;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::gaussian::GaussianBeam;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file;
use lib::output::file::Text;
use lib::sim_region::{Cuboid, VolumeType};
use nalgebra::Vector3;
use specs::{Builder, World};

fn main() {
    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Configure simulation output.
    builder = builder.with(
        file::new::<Position, Text>("pos.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text>("vel.txt".to_string(), 100),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(65.0, Vector3::z()))
        .with(Position::new())
        .build();

    // Create cooling lasers.
    let detuning = -90.0;
    let power = 0.23;
    let radius = 0.0033 / (2.0 * 2.0_f64.sqrt()); // 33mm 1/e^2 diameter

    // Horizontal beams along z
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 5.0,
            direction: Vector3::z(),
        })
        .with(CoolingLight::for_species(
            AtomInfo::strontium(),
            detuning,
            -1.0,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 5.0,
            direction: -Vector3::z(),
        })
        .with(CoolingLight::for_species(
            AtomInfo::strontium(),
            detuning,
            -1.0,
        ))
        .build();

    // Angled vertical beams
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::new(1.0, 1.0, 0.0).normalize(),
        })
        .with(CoolingLight::for_species(
            AtomInfo::strontium(),
            detuning,
            1.0,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::new(1.0, -1.0, 0.0).normalize(),
        })
        .with(CoolingLight::for_species(
            AtomInfo::strontium(),
            detuning,
            1.0,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::new(-1.0, 1.0, 0.0).normalize(),
        })
        .with(CoolingLight::for_species(
            AtomInfo::strontium(),
            detuning,
            1.0,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::new(-1.0, -1.0, 0.0).normalize(),
        })
        .with(CoolingLight::for_species(
            AtomInfo::strontium(),
            detuning,
            1.0,
        ))
        .build();

    // Create an oven.
    // The oven will eject atoms on the first frame and then be deleted.
    let number_to_emit = 100000;
    world
        .create_entity()
        .with(Oven {
            temperature: 776.0,
            aperture: OvenAperture::Circular {
                radius: 0.005,
                thickness: 0.001,
            },
            direction: Vector3::x(),
        })
        .with(Position {
            pos: Vector3::new(-0.083, 0.0, 0.0),
        })
        .with(MassDistribution::new(vec![MassRatio {
            mass: 88.0,
            ratio: 1.0,
        }]))
        .with(AtomInfo::strontium())
        .with(AtomNumberToEmit {
            number: number_to_emit,
        })
        .with(ToBeDestroyed)
        .build();

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-6 });

    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Cuboid {
            half_width: Vector3::new(0.1, 0.01, 0.01),
            vol_type: VolumeType::Inclusive,
        })
        .build();

    // Run the simulation for a number of steps.
    for _i in 0..100000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
}
