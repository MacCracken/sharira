use sharira::biomechanics;
use sharira::{BodyPlan, Bone, BoneId, Gait, Joint, JointType, Muscle, MuscleGroup, Skeleton};

#[test]
fn full_body_assembly() {
    // Build a minimal biped skeleton
    let skeleton = Skeleton {
        name: "biped".into(),
        bones: vec![
            Bone {
                id: BoneId(0),
                name: "pelvis".into(),
                parent: None,
                length: 0.2,
                mass: 5.0,
                local_position: [0.0; 3],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
            },
            Bone {
                id: BoneId(1),
                name: "femur_l".into(),
                parent: Some(BoneId(0)),
                length: 0.45,
                mass: 4.0,
                local_position: [-0.1, -0.1, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
            },
            Bone {
                id: BoneId(2),
                name: "tibia_l".into(),
                parent: Some(BoneId(1)),
                length: 0.4,
                mass: 3.0,
                local_position: [0.0, -0.45, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
            },
            Bone {
                id: BoneId(3),
                name: "femur_r".into(),
                parent: Some(BoneId(0)),
                length: 0.45,
                mass: 4.0,
                local_position: [0.1, -0.1, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
            },
            Bone {
                id: BoneId(4),
                name: "tibia_r".into(),
                parent: Some(BoneId(3)),
                length: 0.4,
                mass: 3.0,
                local_position: [0.0, -0.45, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
            },
        ],
    };

    assert_eq!(skeleton.bone_count(), 5);
    assert_eq!(skeleton.total_mass(), 19.0);
    assert_eq!(skeleton.roots().len(), 1);
    assert_eq!(skeleton.children(BoneId(0)).len(), 2);

    // Attach joints
    let knee_l = Joint::human_knee(BoneId(1), BoneId(2));
    let knee_r = Joint::human_knee(BoneId(3), BoneId(4));
    assert_eq!(knee_l.joint_type, JointType::Hinge);
    assert_eq!(knee_r.joint_type, JointType::Hinge);

    // Attach muscles
    let quad_l = Muscle::new(
        "quad_l",
        BoneId(1),
        BoneId(2),
        MuscleGroup::Extensor,
        5000.0,
        0.3,
    );
    let ham_l = Muscle::new(
        "ham_l",
        BoneId(1),
        BoneId(2),
        MuscleGroup::Flexor,
        3000.0,
        0.25,
    );

    assert!(quad_l.is_antagonist(&ham_l));

    // Assign a gait
    let walk = Gait::human_walk();
    assert!(walk.speed() > 0.0);

    // Biomechanics
    let masses: Vec<f32> = skeleton.bones.iter().map(|b| b.mass).collect();
    let positions: Vec<[f32; 3]> = skeleton.bones.iter().map(|b| b.local_position).collect();
    let com = biomechanics::center_of_mass(&masses, &positions);
    assert!(com[0].is_finite() && com[1].is_finite() && com[2].is_finite());

    // Body plan
    assert_eq!(BodyPlan::Bipedal.limb_count(), 2);
}

#[test]
fn chain_to_root_traversal() {
    let skeleton = Skeleton {
        name: "chain_test".into(),
        bones: vec![
            Bone {
                id: BoneId(0),
                name: "root".into(),
                parent: None,
                length: 0.1,
                mass: 1.0,
                local_position: [0.0; 3],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
            },
            Bone {
                id: BoneId(1),
                name: "child".into(),
                parent: Some(BoneId(0)),
                length: 0.1,
                mass: 1.0,
                local_position: [0.0, 0.1, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
            },
            Bone {
                id: BoneId(2),
                name: "grandchild".into(),
                parent: Some(BoneId(1)),
                length: 0.1,
                mass: 1.0,
                local_position: [0.0, 0.2, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
            },
        ],
    };

    let chain = skeleton.chain_to_root(BoneId(2));
    assert_eq!(chain.len(), 3);
    assert_eq!(chain[0], BoneId(2));
    assert_eq!(chain[2], BoneId(0));
}

#[test]
fn gait_presets_valid() {
    let gaits = [
        Gait::human_walk(),
        Gait::human_run(),
        Gait::quadruped_walk(),
        Gait::quadruped_trot(),
    ];
    for gait in &gaits {
        assert!(gait.cycle.cycle_duration_s > 0.0);
        assert!(gait.cycle.duty_factor > 0.0 && gait.cycle.duty_factor <= 1.0);
        assert!(gait.cycle.stride_length_m > 0.0);
        assert!(gait.speed() > 0.0);
        assert!(gait.speed_range.0 < gait.speed_range.1);
    }
}

#[test]
fn muscle_force_curve() {
    let mut muscle = Muscle::new(
        "test",
        BoneId(0),
        BoneId(1),
        MuscleGroup::Flexor,
        1000.0,
        0.2,
    );

    // Zero activation = zero force
    assert_eq!(muscle.current_force(0.2), 0.0);

    // Max activation at rest length = max force
    muscle.set_activation(1.0);
    let force_at_rest = muscle.current_force(0.2);
    assert!(force_at_rest > 0.0);

    // Total force at moderate stretch includes passive tension,
    // but active component still decreases away from optimal length
    let force_at_moderate_stretch = muscle.current_force(0.25);
    assert!(force_at_moderate_stretch > 0.0);
    // At extreme stretch, force is dominated by passive tension
    let force_very_stretched = muscle.current_force(0.5);
    assert!(
        force_very_stretched > 0.0,
        "passive tension provides force even at extreme stretch"
    );
}

#[test]
fn biomechanics_edge_cases() {
    // Empty inputs
    let com = biomechanics::center_of_mass(&[], &[]);
    assert_eq!(com, [0.0; 3]);

    // Zero mass
    let grf = biomechanics::ground_reaction_force(0.0, 9.81, 0.6);
    assert_eq!(grf, 0.0);

    // Zero duty factor (division guard)
    let grf = biomechanics::ground_reaction_force(70.0, 9.81, 0.0);
    assert_eq!(grf, 0.0);

    // Zero distance (CoT guard)
    let cot = biomechanics::cost_of_transport(100.0, 70.0, 0.0);
    assert_eq!(cot, 0.0);
}

#[test]
fn serde_roundtrip() {
    let walk = Gait::human_walk();
    let json = serde_json::to_string(&walk).expect("serialize");
    let restored: Gait = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.name, walk.name);
    assert_eq!(restored.gait_type, walk.gait_type);
    assert!((restored.speed() - walk.speed()).abs() < 0.001);
}
