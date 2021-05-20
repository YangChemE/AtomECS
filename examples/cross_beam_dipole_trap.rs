//! Loading a Sr cross beam dipole trap from center.
use specs::prelude::*;
extern crate atomecs as lib;
extern crate nalgebra;
use atomecs::laser::cooling::CoolingLight;
use atomecs::laser_cooling::force::EmissionForceOption;
use atomecs::laser_cooling::photons_scattered::ScatteringFluctuationsOption;
use atomecs::magnetic::quadrupole::QuadrupoleField3D;
use lib::atom::Atom;
use lib::atom::{AtomicTransition, Position, Velocity};
use lib::atom_sources::central_creator::CentralCreator;
use lib::atom_sources::emit::AtomNumberToEmit;
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::constant;
use lib::destructor::ToBeDestroyed;
use lib::dipole;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser;
use lib::laser::gaussian::GaussianBeam;
use lib::output::file;
use lib::output::file::{Text, XYZ};
use lib::shapes::Cuboid;
use lib::sim_region::{SimulationVolume, VolumeType};
use nalgebra::Vector3;
use specs::{Builder, RunNow, World};
use std::time::Instant;

fn main() {
    let now = Instant::now();

    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Configure simulation output.
    builder = builder.with(
        file::new::<Position, Text, Atom>("pos_dipole.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text, Atom>("vel_dipole.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Position, XYZ, Atom>("position.xyz".to_string(), 100),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world);

    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(1.0, Vector3::z()))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0e-6),
        })
        .build();

    let detuning = -20.0; //MHz
    let power = 0.01; //W total power of all Lasers together
    let radius = 1.0e-2 / (2.0 * 2.0_f64.sqrt()); // 10mm 1/e^2 diameter

    // Horizontal beams along z
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: Vector3::z(),
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: -Vector3::z(),
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            -1,
        ))
        .build();

    // Angled vertical beams
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(1.0, 1.0, 0.0).normalize(),
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(1.0, -1.0, 0.0).normalize(),
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(-1.0, 1.0, 0.0).normalize(),
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(-1.0, -1.0, 0.0).normalize(),
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world.insert(EmissionForceOption::default());
    world.insert(ScatteringFluctuationsOption::default());

    // Create dipole laser.
    let power = 10.0;
    let e_radius = 60.0e-6 / (2.0_f64.sqrt());

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius: e_radius,
        power: power,
        direction: Vector3::x(),
        rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(&1064.0e-9, &e_radius),
        ellipticity: 0.0,
    };
    world
        .create_entity()
        .with(gaussian_beam)
        .with(laser::dipole_beam::DipoleLight {
            wavelength: 1064.0e-9,
        })
        .with(laser::frame::Frame {
            x_vector: Vector3::y(),
            y_vector: Vector3::z(),
        })
        .build();

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius: e_radius,
        power: power,
        direction: Vector3::y(),
        rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(&1064.0e-9, &e_radius),
        ellipticity: 0.0,
    };
    world
        .create_entity()
        .with(gaussian_beam)
        .with(laser::dipole_beam::DipoleLight {
            wavelength: 1064.0e-9,
        })
        .with(laser::frame::Frame {
            x_vector: Vector3::x(),
            y_vector: Vector3::z(),
        })
        .build();
    // creating the entity that represents the source
    //
    // contains a central creator
    let number_to_emit = 1_000;
    let size_of_cube = 1.0e-5;
    let speed = 0.1; // m/s

    world
        .create_entity()
        .with(CentralCreator::new_uniform_cubic(size_of_cube, speed))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(MassDistribution::new(vec![MassRatio {
            mass: 87.0,
            ratio: 1.0,
        }]))
        .with(AtomicTransition::strontium_red())
        .with(AtomNumberToEmit {
            number: number_to_emit,
        })
        .with(ToBeDestroyed)
        .build();

    // Define timestep
    world.insert(Timestep { delta: 1.0e-5 });
    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Cuboid {
            half_width: Vector3::new(0.0005, 0.0005, 0.0005), //(0.1, 0.01, 0.01)
        })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .build();

    let mut switcher_system =
        dipole::transition_switcher::AttachAtomicDipoleTransitionToAtomsSystem;
    // Run the simulation for a number of steps.
    for _i in 0..100_000 {
        dispatcher.dispatch(&mut world);
        switcher_system.run_now(&world);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
