//! An introduction to the usage of RuBullet.
use std::{thread, time::Duration};

use anyhow::Result;
use misfire::*;
use nalgebra::{Isometry3, Vector3};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut physics_client = PhysicsClient::connect(Mode::Gui)?;

    physics_client.set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")?;
    physics_client.set_gravity(Vector3::new(0.0, 0.0, -10.0));
    physics_client.set_time_step(Duration::from_secs_f64(1. / 120.));
    // physics_client.configure_debug_visualizer(DebugVisualizerFlag::)
    let _plane_id = physics_client.load_urdf("plane100.urdf", None)?;
    let shift = Isometry3::translation(0.0, -0.02, 0.0);
    let visual_shape = physics_client.create_visual_shape(
        GeometricVisualShape::MeshFile {
            filename: PathBuf::from("duck.obj"),
            mesh_scaling: Some(Vector3::from_element(0.1)),
        },
        VisualShapeOptions {
            specular_colors: [0.4, 0.4, 0.],
            frame_offset: shift,
            ..Default::default()
        },
    )?;
    let collision_shape = physics_client.create_collision_shape(
        GeometricCollisionShape::MeshFile {
            filename: PathBuf::from("duck_vhacd.obj"),
            mesh_scaling: Some(Vector3::from_element(0.1)),
            flags: None,
        },
        shift,
    )?;
    let mesh_scaling = Vector3::from_element(0.1);
    let rangex = 1;
    let rangey = 1;
    for i in 0..rangex {
        for j in 0..rangey {
            let _duck = physics_client.create_multi_body(
                collision_shape,
                visual_shape,
                MultiBodyOptions {
                    base_pose: Isometry3::translation(
                        ((-rangex as f64 / 2.) + i as f64) * mesh_scaling[0] * 2.,
                        ((-rangey as f64 / 2.) + j as f64) * mesh_scaling[1] * 2.,
                        1.,
                    ),
                    base_mass: 1.,
                    ..Default::default()
                },
            )?;
        }
    }

    physics_client.set_real_time_simulation(true);
    for _ in 0..10000 {
        thread::sleep(Duration::from_secs_f64(1. / 240.));
    }

    Ok(())
}
