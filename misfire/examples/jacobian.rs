use misfire::Mode::Direct;
use misfire::{BodyId, ControlCommandArray, JointInfo, JointState, PhysicsClient, UrdfOptions};
use nalgebra::{Isometry3, Matrix3xX, Vector3};
use std::time::Duration;

use anyhow::Result;

pub fn set_joint_positions(client: &mut PhysicsClient, robot: BodyId, position: &[f64]) {
    let num_joints = client.get_num_joints(robot);
    assert_eq!(num_joints, position.len());
    let indices = (0..num_joints).collect::<Vec<usize>>();
    let zero_vec = vec![0.; num_joints];
    let position_gains = vec![1.; num_joints];
    let velocity_gains = vec![0.3; num_joints];
    client
        .set_joint_motor_control_array(
            robot,
            indices.as_slice(),
            ControlCommandArray::PositionsWithPd {
                target_positions: position,
                target_velocities: zero_vec.as_slice(),
                position_gains: position_gains.as_slice(),
                velocity_gains: velocity_gains.as_slice(),
            },
            None,
        )
        .unwrap();
}

pub fn get_joint_states(
    client: &mut PhysicsClient,
    robot: BodyId,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let num_joints = client.get_num_joints(robot);
    let indices = (0..num_joints).collect::<Vec<usize>>();
    let joint_states = client.get_joint_states(robot, indices.as_slice()).unwrap();
    let pos = joint_states
        .iter()
        .map(|x| x.joint_position)
        .collect::<Vec<f64>>();
    let vel = joint_states
        .iter()
        .map(|x| x.joint_velocity)
        .collect::<Vec<f64>>();
    let torque = joint_states
        .iter()
        .map(|x| x.joint_motor_torque)
        .collect::<Vec<f64>>();
    (pos, vel, torque)
}

pub fn get_motor_joint_states(
    client: &mut PhysicsClient,
    robot: BodyId,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let num_joints = client.get_num_joints(robot);
    let indices = (0..num_joints).collect::<Vec<usize>>();
    let joint_states = client.get_joint_states(robot, indices.as_slice()).unwrap();
    let joint_infos: Vec<JointInfo> = (0..num_joints)
        .map(|y| client.get_joint_info(robot, y))
        .collect::<Vec<JointInfo>>();
    let joint_states = joint_states
        .iter()
        .zip(joint_infos.iter())
        .filter(|(_, i)| i.q_index > -1)
        .map(|(j, _)| *j)
        .collect::<Vec<JointState>>();
    let pos = joint_states
        .iter()
        .map(|x| x.joint_position)
        .collect::<Vec<f64>>();
    let vel = joint_states
        .iter()
        .map(|x| x.joint_velocity)
        .collect::<Vec<f64>>();
    let torque = joint_states
        .iter()
        .map(|x| x.joint_motor_torque)
        .collect::<Vec<f64>>();
    (pos, vel, torque)
}

pub fn multiply_jacobian(
    client: &mut PhysicsClient,
    robot: BodyId,
    jacobian: &Matrix3xX<f64>,
    vector: &[f64],
) -> Vector3<f64> {
    let mut result = Vector3::new(0., 0., 0.);
    let mut i = 0;
    for c in 0..vector.len() {
        if client.get_joint_info(robot, c).q_index > -1 {
            for r in 0..3 {
                result[r] += jacobian[(r, i)] * vector[c];
            }
            i += 1;
        }
    }
    result
}

fn main() -> Result<()> {
    let delta_t = Duration::from_secs_f64(0.001);
    let mut p = PhysicsClient::connect(Direct).unwrap();
    p.set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")?;
    let gravity_constant = -9.81;
    p.set_time_step(delta_t);
    p.set_gravity(Vector3::new(0., 0., gravity_constant));
    p.load_urdf(
        "plane.urdf",
        UrdfOptions {
            base_transform: Isometry3::translation(0., 0., -0.3),
            ..Default::default()
        },
    )?;

    let kuka_id = p.load_urdf(
        "TwoJointRobot_w_fixedJoints.urdf",
        UrdfOptions {
            use_fixed_base: true,
            ..Default::default()
        },
    )?;
    // let kuka_id = p.load_urdf("kuka_iiwa/model.urdf", UrdfOptions::default())?;
    // let kuka_id = p.load_urdf("kuka_lwr/kuka.urdf", UrdfOptions::default())?;
    let num_joints = p.get_num_joints(kuka_id);
    let kuka_end_effector_index = num_joints - 1;

    set_joint_positions(&mut p, kuka_id, vec![0.1; num_joints].as_slice());
    p.step_simulation()?;

    let (_pos, vel, _torq) = get_joint_states(&mut p, kuka_id);
    let (mpos, _mvel, _mtorq) = get_motor_joint_states(&mut p, kuka_id);
    let result = p.get_link_state(kuka_id, kuka_end_effector_index, true, true)?;

    let zero_vec = vec![0.; mpos.len()];
    let jacobian = p.calculate_jacobian(
        kuka_id,
        kuka_end_effector_index,
        result.local_inertial_pose.translation,
        mpos.as_slice(),
        zero_vec.as_slice(),
        zero_vec.as_slice(),
    )?;
    println!("Link linear velocity of CoM from getLinkState:");
    println!("{:?}", result.get_linear_world_velocity());
    println!("Link linear velocity of CoM from linearJacobian * q_dot:");
    println!(
        "{:?}",
        multiply_jacobian(
            &mut p,
            kuka_id,
            &jacobian.get_linear_jacobian(),
            vel.as_slice()
        )
    );
    println!("Link angular velocity of CoM from getLinkState:");
    println!("{:?}", result.get_angular_world_velocity());
    println!("Link angular velocity of CoM from angularJacobian * q_dot:");
    println!(
        "{:?}",
        multiply_jacobian(
            &mut p,
            kuka_id,
            &jacobian.get_angular_jacobian(),
            vel.as_slice()
        )
    );

    Ok(())
}
