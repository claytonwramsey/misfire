use nalgebra::{DVector, Isometry3, Matrix3xX, Translation3, UnitQuaternion, Vector3};

use anyhow::Result;
use misfire::ControlCommandArray::Torques;
use misfire::Mode::Direct;
use misfire::{
    BodyId, ChangeDynamicsOptions, ConstraintSolverType, ControlCommand, ControlCommandArray,
    DebugVisualizerFlag, Error, InverseKinematicsParametersBuilder, JointFeedbackMode,
    JointInfoFlags, JointType, LoadModelFlags, PhysicsClient, SetPhysicsEngineParameterOptions,
    UrdfOptions,
};
use misfire::{JointInfo, JointState};
use std::f64::consts::PI;
use std::time::Duration;

fn slice_compare(a: &[f64], b: &[f64], thresh: f64) {
    assert_eq!(a.len(), b.len());
    for i in 0..a.len() {
        float_compare(a[i], b[i], thresh);
    }
}

fn float_compare(a: f64, b: f64, thresh: f64) {
    println!("{} {}", a, b);
    assert!((a - b).abs() < thresh);
}
fn slice_compare_f32(a: &[f32], b: &[f32], thresh: f32) {
    assert_eq!(a.len(), b.len());
    for i in 0..a.len() {
        float32_compare(a[i], b[i], thresh);
    }
}

fn float32_compare(a: f32, b: f32, thresh: f32) {
    println!("{} {}", a, b);
    assert!((a - b).abs() < thresh);
}

#[test]
fn test_connect() {
    let _physics_client = PhysicsClient::connect(Direct).unwrap();
}

#[test]
fn test_load_urdf() {
    let mut physics_client = PhysicsClient::connect(Direct).unwrap();
    physics_client
        .set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    let _plane_id = physics_client.load_urdf("plane.urdf", None).unwrap();
}
#[test]
fn test_add_and_remove_bodies() {
    let mut physics_client = PhysicsClient::connect(Direct).unwrap();
    physics_client
        .set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    assert_eq!(physics_client.get_num_bodies(), 0);
    let _plane_id = physics_client.load_urdf("plane.urdf", None).unwrap();
    assert_eq!(physics_client.get_num_bodies(), 1);
    let r2d2 = physics_client.load_urdf("r2d2.urdf", None).unwrap();
    assert_eq!(physics_client.get_num_bodies(), 2);
    physics_client.remove_body(r2d2);
    assert_eq!(physics_client.get_num_bodies(), 1);
    physics_client.reset_simulation();
    assert_eq!(physics_client.get_num_bodies(), 0);
}
#[test]
fn test_get_and_reset_base_transformation() {
    let mut physics_client = PhysicsClient::connect(Direct).unwrap();
    physics_client
        .set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    let r2d2 = physics_client.load_urdf("r2d2.urdf", None).unwrap();
    let desired_transform = Isometry3::from_parts(
        Translation3::new(0.2, 0.3, 0.4),
        UnitQuaternion::from_euler_angles(0.1, 0.2, 0.3),
    );
    physics_client.reset_base_transform(r2d2, desired_transform);
    let actual_transform = physics_client.get_base_transform(r2d2).unwrap();
    slice_compare(
        desired_transform.translation.vector.as_slice(),
        actual_transform.translation.vector.as_slice(),
        1e-5,
    );
    slice_compare(
        desired_transform.rotation.coords.as_slice(),
        actual_transform.rotation.coords.as_slice(),
        1e-5,
    );

    let desired_transform = Isometry3::from_parts(
        Translation3::new(3.7, -0.23, 10.4),
        UnitQuaternion::from_euler_angles(1.1, -0.2, 2.3),
    );
    physics_client.reset_base_transform(r2d2, desired_transform);
    let actual_transform = physics_client.get_base_transform(r2d2).unwrap();
    slice_compare(
        desired_transform.translation.vector.as_slice(),
        actual_transform.translation.vector.as_slice(),
        1e-5,
    );
    slice_compare(
        desired_transform.rotation.coords.as_slice(),
        actual_transform.rotation.coords.as_slice(),
        1e-5,
    );
}
#[test]
fn test_get_body_info() {
    let mut physics_client = PhysicsClient::connect(Direct).unwrap();
    physics_client
        .set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    let r2d2 = physics_client.load_urdf("r2d2.urdf", None).unwrap();
    let body_info = physics_client.get_body_info(r2d2).unwrap();
    assert_eq!(body_info.base_name.as_str(), "base_link");
    assert_eq!(body_info.body_name.as_str(), "physics");
}
#[test]
#[should_panic]
fn test_get_joint_info_index_out_of_range() {
    let mut physics_client = PhysicsClient::connect(Direct).unwrap();
    physics_client
        .set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    let r2d2 = physics_client.load_urdf("r2d2.urdf", None).unwrap();
    let num_joints = physics_client.get_num_joints(r2d2);
    let joint_info = physics_client.get_joint_info(r2d2, num_joints);
    println!("{:?}", joint_info);
}

#[test]
// tests a fixed joint and a revolute joint
fn test_get_joint_info() {
    let mut physics_client = PhysicsClient::connect(Direct).unwrap();
    physics_client
        .set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    let r2d2 = physics_client.load_urdf("r2d2.urdf", None).unwrap();
    let joint_info = physics_client.get_joint_info(r2d2, 1);
    assert_eq!(1, joint_info.joint_index);
    assert_eq!("right_base_joint", joint_info.joint_name);

    assert_eq!(JointType::Fixed, joint_info.joint_type);
    assert_eq!(-1, joint_info.q_index);
    assert_eq!(-1, joint_info.u_index);
    assert!(joint_info.flags.is_empty());
    float_compare(0., joint_info.joint_damping, 1e-10);
    float_compare(0., joint_info.joint_friction, 1e-10);
    float_compare(0., joint_info.joint_lower_limit, 1e-10);
    float_compare(-1., joint_info.joint_upper_limit, 1e-10);
    float_compare(0., joint_info.joint_max_force, 1e-10);
    float_compare(0., joint_info.joint_max_velocity, 1e-10);
    assert_eq!("right_base", joint_info.link_name);
    slice_compare(&[0.; 3], joint_info.joint_axis.as_slice(), 1e-10);
    println!("{}", joint_info.parent_frame_pose.rotation.quaternion());
    slice_compare(
        &[0.2999999996780742, 0., -1.389_803_846_394_421_6e-5],
        joint_info.parent_frame_pose.translation.vector.as_slice(),
        1e-7,
    );
    slice_compare(
        &[0.0, 0.7070904020014416, 0.0, 0.7071231599922604],
        joint_info.parent_frame_pose.rotation.coords.as_slice(),
        1e-7,
    );
    assert_eq!(0, joint_info.parent_index.unwrap());
    let joint_info = physics_client.get_joint_info(r2d2, 2);
    assert_eq!(2, joint_info.joint_index);
    assert_eq!("right_front_wheel_joint", joint_info.joint_name);

    assert_eq!(JointType::Revolute, joint_info.joint_type);
    assert_eq!(7, joint_info.q_index);
    assert_eq!(6, joint_info.u_index);
    assert_eq!(JointInfoFlags::JOINT_CHANGE_MAX_FORCE, joint_info.flags);
    float_compare(0., joint_info.joint_damping, 1e-10);
    float_compare(0., joint_info.joint_friction, 1e-10);
    float_compare(0., joint_info.joint_lower_limit, 1e-10);
    float_compare(-1., joint_info.joint_upper_limit, 1e-10);
    float_compare(100., joint_info.joint_max_force, 1e-10);
    float_compare(100., joint_info.joint_max_velocity, 1e-10);
    assert_eq!("right_front_wheel", joint_info.link_name);
    slice_compare(&[0., 0., 1.], joint_info.joint_axis.as_slice(), 1e-10);
    println!("{}", joint_info.parent_frame_pose.rotation.quaternion());
    slice_compare(
        &[0.0, 0.133333333333, -0.085],
        joint_info.parent_frame_pose.translation.vector.as_slice(),
        1e-7,
    );
    slice_compare(
        &[0.0, -0.7070904020014416, 0.0, 0.7071231599922604],
        joint_info.parent_frame_pose.rotation.coords.as_slice(),
        1e-7,
    );
    assert_eq!(1, joint_info.parent_index.unwrap());
}

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
        .filter(|(_j, i)| i.q_index > -1)
        .map(|(j, _i)| *j)
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

#[test]
fn test_jacobian() {
    let delta_t = Duration::from_secs_f64(0.001);
    let mut p = PhysicsClient::connect(Direct).unwrap();
    p.set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    p.set_time_step(delta_t);

    let kuka_id = p
        .load_urdf(
            "TwoJointRobot_w_fixedJoints.urdf",
            UrdfOptions {
                use_fixed_base: true,
                ..Default::default()
            },
        )
        .unwrap();

    let num_joints = p.get_num_joints(kuka_id);
    let kuka_end_effector_index = num_joints - 1;
    set_joint_positions(&mut p, kuka_id, vec![0.1; num_joints].as_slice());
    p.step_simulation().unwrap();

    let (_pos, vel, _torq) = get_joint_states(&mut p, kuka_id);
    let (mpos, _mvel, _mtorq) = get_motor_joint_states(&mut p, kuka_id);
    let result = p
        .get_link_state(kuka_id, kuka_end_effector_index, true, true)
        .unwrap();
    println!("{:?}", vel);
    let zero_vec = vec![0.; mpos.len()];
    let jacobian = p
        .calculate_jacobian(
            kuka_id,
            kuka_end_effector_index,
            result.local_inertial_pose.translation,
            mpos.as_slice(),
            zero_vec.as_slice(),
            zero_vec.as_slice(),
        )
        .unwrap();
    let q_dot: DVector<f64> = DVector::from_vec(_mvel);
    // println!("aaa{:?}", vel);
    let cartesian_velocity = jacobian.clone() * q_dot;
    println!("{:?}", cartesian_velocity);
    let linear_vel = multiply_jacobian(
        &mut p,
        kuka_id,
        &jacobian.get_linear_jacobian(),
        vel.as_slice(),
    );
    let angular_vel = multiply_jacobian(
        &mut p,
        kuka_id,
        &jacobian.get_angular_jacobian(),
        vel.as_slice(),
    );
    slice_compare(
        cartesian_velocity.get_linear_velocity().as_slice(),
        linear_vel.as_slice(),
        1e-10,
    );
    slice_compare(
        cartesian_velocity.get_angular_velocity().as_slice(),
        angular_vel.as_slice(),
        1e-10,
    );
    let target_linear_jacobian = [
        -0.1618321740829912,
        1.9909341219607504,
        0.0,
        -0.13073095940453022,
        0.991417881749755,
        0.0,
    ];
    for (i, j) in jacobian
        .get_linear_jacobian()
        .as_slice()
        .iter()
        .zip(target_linear_jacobian.iter())
    {
        println!("{} {}", i, j);
        assert!((i - j).abs() < 1e-6);
    }
    let target_angluar_jacobian = [0., 0., 1.0, 0.0, 0., 1.];
    for (i, j) in jacobian
        .get_angular_jacobian()
        .as_slice()
        .iter()
        .zip(target_angluar_jacobian.iter())
    {
        println!("{} {}", i, j);
        assert!((i - j).abs() < 1e-6);
    }
}

#[test]
fn test_get_link_state() {
    let delta_t = Duration::from_secs_f64(0.001);
    let mut p = PhysicsClient::connect(Direct).unwrap();
    p.set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();

    p.set_time_step(delta_t);

    let kuka_id = p
        .load_urdf(
            "TwoJointRobot_w_fixedJoints.urdf",
            UrdfOptions {
                use_fixed_base: true,
                ..Default::default()
            },
        )
        .unwrap();

    let num_joints = p.get_num_joints(kuka_id);
    let kuka_end_effector_index = num_joints - 1;
    set_joint_positions(&mut p, kuka_id, vec![0.1; num_joints].as_slice());
    p.step_simulation().unwrap();

    let result = p
        .get_link_state(kuka_id, kuka_end_effector_index, true, true)
        .unwrap();
    let m_world_position = [1.9909341219607506, 0.1618321740829912, 0.12500000000000003];
    let m_world_orientation = [-0.0, -0.0, 0.06550617623646283, 0.997852163837348];
    let m_local_inertial_position = [0.; 3];
    let m_local_inertial_orientation = [0., 0., 0., 1.];
    let m_world_link_frame_position = [1.990934133529663, 0.16183216869831085, 0.125];
    let m_world_link_frame_orientation = [0.0, 0.0, 0.06550618261098862, 0.9978521466255188];
    let m_world_linear_velocity = [-18.107084901818524, 161.0722445230232, 0.0];
    let m_world_angular_velocity = [0.0, 0.0, 131.10623082146793];
    slice_compare(
        result.world_pose.translation.vector.as_slice(),
        &m_world_position,
        1e-6,
    );
    slice_compare(
        result.world_pose.rotation.coords.as_slice(),
        &m_world_orientation,
        1e-6,
    );
    slice_compare(
        result.local_inertial_pose.translation.vector.as_slice(),
        &m_local_inertial_position,
        1e-6,
    );
    slice_compare(
        result.local_inertial_pose.rotation.coords.as_slice(),
        &m_local_inertial_orientation,
        1e-6,
    );
    slice_compare(
        result.world_link_frame_pose.translation.vector.as_slice(),
        &m_world_link_frame_position,
        1e-6,
    );
    slice_compare(
        result.world_link_frame_pose.rotation.coords.as_slice(),
        &m_world_link_frame_orientation,
        1e-6,
    );
    slice_compare(
        result.get_linear_world_velocity().unwrap().as_slice(),
        &m_world_linear_velocity,
        1e-5,
    );
    slice_compare(
        result.get_angular_world_velocity().unwrap().as_slice(),
        &m_world_angular_velocity,
        1e-6,
    );
    let link_states = p
        .get_link_states(kuka_id, &[kuka_end_effector_index], true, true)
        .unwrap();
    let link_state_from_link_states = link_states.first().unwrap();
    slice_compare(
        link_state_from_link_states
            .world_link_frame_pose
            .to_homogeneous()
            .as_slice(),
        result.world_link_frame_pose.to_homogeneous().as_slice(),
        1e-10,
    );
}

#[test]
pub fn inverse_dynamics_test() {
    let target_torque = [
        [
            2.7890393556850084,
            -4.35022826384923,
            -9.806091463156854,
            -11.36404798189885,
            -8.452873740133834,
            -2.4533327096931083,
            4.292827558364013,
            9.478755361855157,
            11.229787344270306,
            8.697705289653415,
        ],
        [
            1.5324022942490911,
            -0.3981591800958851,
            -2.1556396779135447,
            -3.0122972444815823,
            -2.612875295201555,
            -1.1830210438812325,
            0.6634457473498473,
            2.2739591995615016,
            3.11408342881574,
            2.85137408903459,
        ],
    ];
    let delta_t = Duration::from_secs_f64(0.1);
    let mut physics_client = PhysicsClient::connect(Direct).unwrap();
    physics_client
        .set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    physics_client.set_time_step(delta_t);
    let id_revolute_joints = [0, 3];
    let id_robot = physics_client
        .load_urdf(
            "TwoJointRobot_w_fixedJoints.urdf",
            UrdfOptions {
                use_fixed_base: true,
                ..Default::default()
            },
        )
        .unwrap();
    physics_client.change_dynamics(
        id_robot,
        None,
        ChangeDynamicsOptions {
            linear_damping: Some(0.),
            angular_damping: Some(0.),
            ..Default::default()
        },
    );
    physics_client
        .set_joint_motor_control_array(
            id_robot,
            &id_revolute_joints,
            ControlCommandArray::Velocities(&[0., 0.]),
            Some(&[0., 0.]),
        )
        .unwrap();

    // Target Positions:
    let start = 0.;
    let end = 1.;

    let steps = ((end - start) / delta_t.as_secs_f64()) as usize;
    let mut t = vec![0.; steps];

    let mut q_pos_desired = vec![vec![0.; steps]; 2];
    let mut q_vel_desired = vec![vec![0.; steps]; 2];
    let mut q_acc_desired = vec![vec![0.; steps]; 2];

    for s in 0..steps {
        t[s] = start + s as f64 * delta_t.as_secs_f64();
        q_pos_desired[0][s] = 1. / (2. * PI) * f64::sin(2. * PI * t[s]) - t[s];
        q_pos_desired[1][s] = -1. / (2. * PI) * f64::sin(2. * PI * t[s]) - 1.;

        q_vel_desired[0][s] = f64::cos(2. * PI * t[s]) - 1.;
        q_vel_desired[1][s] = f64::sin(2. * PI * t[s]);

        q_acc_desired[0][s] = -2. * PI * f64::sin(2. * PI * t[s]);
        q_acc_desired[1][s] = 2. * PI * f64::cos(2. * PI * t[s]);
    }
    let mut q_pos = vec![vec![0.; steps]; 2];
    let mut q_vel = vec![vec![0.; steps]; 2];
    let mut q_tor = vec![vec![0.; steps]; 2];

    for i in 0..t.len() {
        let joint_states = physics_client.get_joint_states(id_robot, &[0, 3]).unwrap();
        q_pos[0][1] = joint_states[0].joint_position;
        let a = joint_states[1].joint_position;
        q_pos[1][i] = a;

        q_vel[0][i] = joint_states[0].joint_velocity;
        q_vel[1][i] = joint_states[1].joint_velocity;

        let obj_pos = [q_pos[0][i], q_pos[1][i]];
        let obj_vel = [q_vel[0][i], q_vel[1][i]];
        let obj_acc = [q_acc_desired[0][i], q_acc_desired[1][i]];

        let torque = physics_client
            .calculate_inverse_dynamics(id_robot, &obj_pos, &obj_vel, &obj_acc)
            .unwrap();

        q_tor[0][i] = torque[0];
        q_tor[1][i] = torque[1];

        physics_client
            .set_joint_motor_control_array(id_robot, &id_revolute_joints, Torques(&torque), None)
            .unwrap();
        physics_client.step_simulation().unwrap();
    }
    slice_compare(q_tor[0].as_slice(), &target_torque[0], 1e-10);
}

#[test]
fn test_mass_matrix_and_inverse_kinematics() -> Result<()> {
    let mut physics_client = PhysicsClient::connect(Direct)?;
    physics_client.configure_debug_visualizer(DebugVisualizerFlag::CovEnableYAxisUp, true);
    physics_client.set_time_step(Duration::from_secs_f64(1. / 60.));
    physics_client.set_gravity(Vector3::new(0.0, -9.8, 0.));

    println!("a");
    let mut panda = PandaSim::new(&mut physics_client, Vector3::zeros())?;
    println!("b");
    panda.step(&mut physics_client);

    Ok(())
}

pub struct PandaSim {
    pub offset: Vector3<f64>,
    pub id: BodyId,
    pub t: Duration,
}

impl PandaSim {
    const INITIAL_JOINT_POSITIONS: [f64; 9] =
        [0.98, 0.458, 0.31, -2.24, -0.30, 2.66, 2.32, 0.02, 0.02];
    const PANDA_NUM_DOFS: usize = 7;
    const PANDA_END_EFFECTOR_INDEX: usize = 11;
    pub fn new(client: &mut PhysicsClient, offset: Vector3<f64>) -> Result<Self, Error> {
        client.set_additional_search_path(
            "../misfire-sys/bullet3/libbullet3/examples/pybullet/gym/pybullet_data",
        )?;
        let cube_start_position = Isometry3::new(
            Vector3::new(0., 0., 0.),
            UnitQuaternion::from_euler_angles(-PI / 2., 0., 0.).scaled_axis(),
        );

        let urdf_options = UrdfOptions {
            use_fixed_base: true,
            base_transform: cube_start_position,
            flags: LoadModelFlags::URDF_ENABLE_CACHED_GRAPHICS_SHAPES,
            ..Default::default()
        };
        let panda_id = client.load_urdf("franka_panda/panda.urdf", urdf_options)?;
        client.change_dynamics(
            panda_id,
            None,
            ChangeDynamicsOptions {
                linear_damping: Some(0.),
                angular_damping: Some(0.),
                ..Default::default()
            },
        );
        let mut index = 0;
        for i in 0..client.get_num_joints(panda_id) {
            let info = client.get_joint_info(panda_id, i);
            if info.joint_type == JointType::Revolute || info.joint_type == JointType::Prismatic {
                client.reset_joint_state(
                    panda_id,
                    i,
                    PandaSim::INITIAL_JOINT_POSITIONS[index],
                    None,
                )?;
                index += 1;
            }
        }
        let t = Duration::new(0, 0);
        Ok(PandaSim {
            offset,
            id: panda_id,
            t,
        })
    }
    pub fn step(&mut self, client: &mut PhysicsClient) {
        let t = self.t.as_secs_f64();
        self.t += Duration::from_secs_f64(1. / 60.);

        let pose = Isometry3::from_parts(
            Translation3::new(
                0.2 * f64::sin(1.5 * t),
                0.044,
                -0.6 + 0.1 * f64::cos(1.5 * t),
            ),
            UnitQuaternion::<f64>::from_euler_angles(PI / 2., 0., 0.),
        );
        let inverse_kinematics_parameters =
            InverseKinematicsParametersBuilder::new(PandaSim::PANDA_END_EFFECTOR_INDEX, &pose)
                .set_max_num_iterations(5)
                .build();
        let joint_poses = client
            .calculate_inverse_kinematics(self.id, inverse_kinematics_parameters)
            .unwrap();

        for i in 0..PandaSim::PANDA_NUM_DOFS {
            client.set_joint_motor_control(
                self.id,
                i,
                ControlCommand::Position(joint_poses[i]),
                Some(240. * 5.),
            );
        }
        let target_mass_matrix = [
            1.2851012047449573,
            0.019937918309349063,
            1.099094168104455,
            -0.13992071941819356,
            -0.04530258995812824,
            0.015010766618204326,
            -0.00419273685297444,
            -0.004045701114304651,
            0.004045701114304651,
            0.019937918309349063,
            1.4200408634854977,
            -0.13973846625926817,
            -0.6766534143669977,
            -0.011150292631844208,
            -0.11222097289908575,
            7.963_273_507_305_265e-5,
            0.021288781819096075,
            -0.021288781819096075,
            1.099094168104455,
            -0.13973846625926817,
            1.0599485744387636,
            -0.01927738076755258,
            -0.03898176486608705,
            0.0367718209930567,
            -0.0038394168220030464,
            -0.008321730809750807,
            0.008321730809750807,
            -0.13992071941819356,
            -0.6766534143669977,
            -0.01927738076755258,
            0.8883254282752625,
            0.03919655691643165,
            0.13355872722768167,
            0.0005209277856703768,
            -0.04891378328717881,
            0.04891378328717881,
            -0.04530258995812824,
            -0.011150292631844208,
            -0.03898176486608705,
            0.03919655691643165,
            0.028341271915447625,
            -0.0001434455846943853,
            0.0037745850335795,
            1.888_537_423_501_440_3e-5,
            -1.888_537_423_501_440_3e-5,
            0.015010766618204326,
            -0.11222097289908575,
            0.0367718209930567,
            0.13355872722768167,
            -0.0001434455846943853,
            0.047402314149303876,
            2.053797643165216e-14,
            -0.018417574890008004,
            0.018417574890008004,
            -0.00419273685297444,
            7.963_273_507_305_265e-5,
            -0.0038394168220030464,
            0.0005209277856703768,
            0.0037745850335795,
            2.053797643165216e-14,
            0.004194301009161442,
            0.0,
            0.0,
            -0.004045701114304651,
            0.021288781819096075,
            -0.008321730809750807,
            -0.04891378328717881,
            1.888_537_423_501_440_3e-5,
            -0.018417574890008004,
            0.0,
            0.1,
            0.0,
            0.004045701114304651,
            -0.021288781819096075,
            0.008321730809750807,
            0.04891378328717881,
            -1.888_537_423_501_440_3e-5,
            0.018417574890008004,
            0.0,
            0.0,
            0.1,
        ];
        let target_joint_poses = [
            1.1029851000632531,
            0.43354557662855453,
            0.3608104666320187,
            -2.3105861116521096,
            -0.2888395010735958,
            2.6904095021250938,
            2.4711777602235387,
            0.02,
            0.02,
        ];
        let mass = client
            .calculate_mass_matrix(self.id, joint_poses.as_slice())
            .unwrap();
        slice_compare(joint_poses.as_slice(), &target_joint_poses, 1e-6);
        slice_compare(mass.as_slice(), &target_mass_matrix, 1e-6);
    }
}
#[test]
fn compute_view_matrix_test() {
    let eye_position = [1.; 3];
    let target_position = [1., 0., 0.];
    let up_vector = [0., 1., 0.];
    let view_matrix = PhysicsClient::compute_view_matrix(eye_position, target_position, up_vector);
    let desired_matrix = [
        0.99999994,
        0.0,
        -0.0,
        0.0,
        -0.0,
        0.7071067,
        0.70710677,
        0.0,
        0.0,
        -0.7071067,
        0.70710677,
        0.0,
        -0.99999994,
        -0.0,
        -1.4142135,
        1.0,
    ];
    slice_compare_f32(view_matrix.as_slice(), &desired_matrix, 1e-7);
}
#[test]
fn compute_view_matrix_from_yaw_pitch_roll_test() {
    let target_position = [1., 0., 0.];
    let view_matrix = PhysicsClient::compute_view_matrix_from_yaw_pitch_roll(
        target_position,
        0.6,
        0.2,
        0.3,
        0.5,
        false,
    );
    let desired_matrix = [
        -0.999_994,
        -1.827_692_4e-5,
        -0.003_490_646_6,
        0.0,
        2.237_357e-10,
        0.999_986_4,
        -0.005_235_963_5,
        0.0,
        0.003_490_694_3,
        -0.005_235_932,
        -0.999_980_3,
        0.0,
        0.999_994,
        1.827_720_6e-5,
        -0.596_509_4,
        1.0,
    ];
    slice_compare_f32(view_matrix.as_slice(), &desired_matrix, 1e-7);
}
#[test]
fn compute_projection_matrix_fov_test() {
    let projection_matrix = PhysicsClient::compute_projection_matrix_fov(0.4, 0.6, 0.2, 0.6);
    let desired_matrix = [
        477.462_86,
        0.0,
        0.0,
        0.0,
        0.0,
        286.477_72,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.999_999_9,
        -1.0,
        0.0,
        0.0,
        -0.599_999_96,
        0.0,
    ];
    slice_compare_f32(projection_matrix.as_slice(), &desired_matrix, 1e-7);
}
#[test]
fn compute_projection_matrix_test() {
    let projection_matrix = PhysicsClient::compute_projection_matrix(0.1, 0.2, 0.3, 0.4, 0.2, 0.6);
    let desired_matrix = [
        4.0,
        0.0,
        0.0,
        0.0,
        0.0,
        4.000_000_5,
        0.0,
        0.0,
        3.0,
        7.000_001,
        -1.999_999_9,
        -1.0,
        0.0,
        0.0,
        -0.599_999_96,
        0.0,
    ];
    slice_compare_f32(projection_matrix.as_slice(), &desired_matrix, 1e-7);
}
#[test]
fn save_and_restore_test() {
    let mut client = PhysicsClient::connect(Direct).unwrap();
    client
        .set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    let cube = client.load_urdf("cube.urdf", None).unwrap();
    let cube_pose_start = client.get_base_transform(cube).unwrap();
    slice_compare(
        cube_pose_start.translation.vector.as_slice(),
        Vector3::zeros().as_slice(),
        1e-10,
    );
    slice_compare(
        cube_pose_start.rotation.coords.as_slice(),
        UnitQuaternion::identity().coords.as_slice(),
        1e-10,
    );
    let start_state = client.save_state().unwrap();
    let transform = Isometry3::translation(1., 1., 1.);
    client.reset_base_transform(cube, transform);
    let cube_pose_end = client.get_base_transform(cube).unwrap();
    slice_compare(
        cube_pose_end.translation.vector.as_slice(),
        transform.translation.vector.as_slice(),
        1e-10,
    );
    slice_compare(
        cube_pose_end.rotation.coords.as_slice(),
        transform.rotation.coords.as_slice(),
        1e-10,
    );

    client.restore_state(start_state).unwrap();
    let cube_pose_restored = client.get_base_transform(cube).unwrap();
    slice_compare(
        cube_pose_restored.translation.vector.as_slice(),
        Vector3::zeros().as_slice(),
        1e-10,
    );
    slice_compare(
        cube_pose_restored.rotation.coords.as_slice(),
        UnitQuaternion::identity().coords.as_slice(),
        1e-10,
    );

    client.remove_state(start_state);
}
#[test]
fn save_and_restore_from_file_test() {
    let mut client = PhysicsClient::connect(Direct).unwrap();
    client
        .set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    let cube = client.load_urdf("cube.urdf", None).unwrap();
    let cube_pose_start = client.get_base_transform(cube).unwrap();
    slice_compare(
        cube_pose_start.translation.vector.as_slice(),
        Vector3::zeros().as_slice(),
        1e-10,
    );
    slice_compare(
        cube_pose_start.rotation.coords.as_slice(),
        UnitQuaternion::identity().coords.as_slice(),
        1e-10,
    );
    client
        .save_bullet("save_and_restore_from_file_test.bullet")
        .unwrap();
    let transform = Isometry3::translation(1., 1., 1.);
    client.reset_base_transform(cube, transform);
    let cube_pose_end = client.get_base_transform(cube).unwrap();
    slice_compare(
        cube_pose_end.translation.vector.as_slice(),
        transform.translation.vector.as_slice(),
        1e-10,
    );
    slice_compare(
        cube_pose_end.rotation.coords.as_slice(),
        transform.rotation.coords.as_slice(),
        1e-10,
    );

    client
        .restore_state_from_file("save_and_restore_from_file_test.bullet")
        .unwrap();
    let cube_pose_restored = client.get_base_transform(cube).unwrap();
    slice_compare(
        cube_pose_restored.translation.vector.as_slice(),
        Vector3::zeros().as_slice(),
        1e-10,
    );
    slice_compare(
        cube_pose_restored.rotation.coords.as_slice(),
        UnitQuaternion::identity().coords.as_slice(),
        1e-10,
    );

    std::fs::remove_file("save_and_restore_from_file_test.bullet").unwrap();
}

#[test]
fn load_bullet_test() {
    let mut client = PhysicsClient::connect(Direct).unwrap();
    client
        .set_additional_search_path("../misfire-sys/bullet3/libbullet3/data")
        .unwrap();
    let bodies = client.load_bullet("spider.bullet").unwrap();
    assert_eq!(bodies.len(), 27);
}

#[test]
fn set_and_get_physics_engine_parameters() {
    let mut client = PhysicsClient::connect(Direct).unwrap();
    let _params = client.get_physics_engine_parameters().unwrap();
    let b = true;
    let f = 0.3;
    let u = 5;
    let dur = Duration::from_secs_f64(0.11);
    client.set_physics_engine_parameter(SetPhysicsEngineParameterOptions {
        fixed_time_step: Some(dur),
        num_solver_iterations: Some(u),
        use_split_impulse: Some(b),
        split_impulse_penetration_threshold: Some(f),
        num_sub_steps: Some(u),
        collision_filter_mode: Some(u),
        contact_breaking_threshold: Some(f),
        max_num_cmd_per_1_ms: Some(u as i32),
        enable_file_caching: Some(b),
        restitution_velocity_threshold: Some(f),
        erp: Some(f),
        contact_erp: Some(f),
        friction_erp: Some(f),
        enable_cone_friction: Some(b),
        deterministic_overlapping_pairs: Some(b),
        allowed_ccd_penetration: Some(f),
        joint_feedback_mode: Some(JointFeedbackMode::WorldSpace),
        solver_residual_threshold: Some(f),
        contact_slop: Some(f),
        enable_sat: Some(b),
        constraint_solver_type: Some(ConstraintSolverType::Dantzig),
        global_cfm: Some(f),
        minimum_solver_island_size: Some(u),
        report_solver_analytics: Some(b),
        warm_starting_factor: Some(f),
        sparse_sdf_voxel_size: Some(f),
        num_non_contact_inner_iterations: Some(u),
    });
    let params = client.get_physics_engine_parameters().unwrap();
    assert_eq!(params.fixed_time_step, dur);
    assert_eq!(params.num_solver_iterations, u);
    assert_eq!(params.use_split_impulse, b);
    assert_eq!(params.split_impulse_penetration_threshold, f);
    assert_eq!(params.num_sub_steps, u);
    assert_eq!(params.collision_filter_mode, u);
    assert_eq!(params.contact_breaking_threshold, f);
    assert_eq!(params.enable_file_caching, b);
    assert_eq!(params.restitution_velocity_threshold, f);
    assert_eq!(params.erp, f);
    assert_eq!(params.contact_erp, f);
    assert_eq!(params.friction_erp, f);
    assert_eq!(params.enable_cone_friction, b);
    assert_eq!(params.deterministic_overlapping_pairs, b);
    assert_eq!(params.allowed_ccd_penetration, f);
    assert_eq!(params.joint_feedback_mode, JointFeedbackMode::WorldSpace);
    assert_eq!(params.solver_residual_threshold, f);
    assert_eq!(params.contact_slop, f);
    assert_eq!(params.enable_sat, b);
    // assert_eq!(params.constraint_solver_type, ConstraintSolverType::Dantzig);// bug in bullet3
    assert_eq!(params.global_cfm, f);
    // assert_eq!(params.minimum_solver_island_size, u);// bug in bullet3
    // assert_eq!(params.report_solver_analytics, b);// bug in bullet3
    // assert_eq!(params.warm_starting_factor, f);// bug in bullet3
    // assert_eq!(params.sparse_sdf_voxel_size, f);// bug in bullet3
    assert_eq!(params.num_non_contact_inner_iterations, u);
}
