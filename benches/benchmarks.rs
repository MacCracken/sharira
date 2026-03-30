use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hisab::{Quat, Vec3};
use sharira::biomechanics;
use sharira::{
    AllometricParams, Body, BodyPlan, Bone, BoneId, FatigueState, Gait, GaitController, IKChain,
    IKTarget, Joint, Morphology, Muscle, MuscleGroup, Pose, Skeleton, allometric_skeleton,
    apply_morphology, forward_kinematics, solve_fabrik, solve_two_bone,
};

// ── Helpers ────────────────────────────────────────────────────────────────

fn build_skeleton(bone_count: u16) -> Skeleton {
    let mut skeleton = Skeleton::new("bench");
    skeleton.add_bone(Bone::new(BoneId(0), "root", 0.2, 5.0, None));
    for i in 1..bone_count {
        skeleton.add_bone(
            Bone::new(
                BoneId(i),
                format!("bone_{i}"),
                0.1,
                1.0,
                Some(BoneId(i - 1)),
            )
            .with_position(Vec3::new(0.0, 0.1, 0.0)),
        );
    }
    skeleton
}

fn build_two_bone_chain() -> (Skeleton, IKChain) {
    let mut skeleton = Skeleton::new("arm");
    skeleton.add_bone(Bone::new(BoneId(0), "shoulder", 0.0, 1.0, None));
    skeleton.add_bone(
        Bone::new(BoneId(1), "upper_arm", 0.3, 1.0, Some(BoneId(0)))
            .with_position(Vec3::new(0.0, -0.3, 0.0)),
    );
    skeleton.add_bone(
        Bone::new(BoneId(2), "forearm", 0.25, 0.8, Some(BoneId(1)))
            .with_position(Vec3::new(0.0, -0.25, 0.0)),
    );
    let chain = IKChain::new(
        vec![BoneId(0), BoneId(1), BoneId(2)],
        vec![
            Joint::human_shoulder(BoneId(0), BoneId(1)),
            Joint::human_knee(BoneId(1), BoneId(2)),
        ],
    );
    (skeleton, chain)
}

// ── Skeleton ───────────────────────────────────────────────────────────────

fn bench_skeleton_total_mass(c: &mut Criterion) {
    let skeleton = build_skeleton(100);
    c.bench_function("skeleton_total_mass_100", |b| {
        b.iter(|| black_box(skeleton.total_mass()))
    });
}

fn bench_skeleton_find_bone(c: &mut Criterion) {
    let skeleton = build_skeleton(100);
    c.bench_function("skeleton_find_bone_100", |b| {
        b.iter(|| black_box(skeleton.find_bone("bone_50")))
    });
}

fn bench_skeleton_chain_to_root(c: &mut Criterion) {
    let skeleton = build_skeleton(100);
    c.bench_function("skeleton_chain_to_root_100", |b| {
        b.iter(|| black_box(skeleton.chain_to_root(BoneId(99))))
    });
}

fn bench_skeleton_children(c: &mut Criterion) {
    let skeleton = build_skeleton(100);
    c.bench_function("skeleton_children_100", |b| {
        b.iter(|| black_box(skeleton.children(BoneId(0))))
    });
}

// ── Muscle ─────────────────────────────────────────────────────────────────

fn bench_muscle_force(c: &mut Criterion) {
    let mut muscle = Muscle::new(
        "bench",
        BoneId(0),
        BoneId(1),
        MuscleGroup::Flexor,
        1000.0,
        0.3,
    );
    muscle.set_activation(0.8);
    c.bench_function("muscle_current_force", |b| {
        b.iter(|| black_box(muscle.current_force(black_box(0.25))))
    });
}

fn bench_muscle_force_at(c: &mut Criterion) {
    let mut muscle = Muscle::new(
        "bench",
        BoneId(0),
        BoneId(1),
        MuscleGroup::Flexor,
        1000.0,
        0.3,
    );
    muscle.set_activation(0.8);
    c.bench_function("muscle_force_at_with_velocity", |b| {
        b.iter(|| black_box(muscle.force_at(black_box(0.25), black_box(-2.0))))
    });
}

fn bench_muscle_activation_update(c: &mut Criterion) {
    let mut muscle = Muscle::new(
        "bench",
        BoneId(0),
        BoneId(1),
        MuscleGroup::Flexor,
        1000.0,
        0.3,
    );
    muscle.set_excitation(0.8);
    c.bench_function("muscle_activation_update", |b| {
        b.iter(|| {
            muscle.update_activation(black_box(0.001));
        })
    });
}

// ── Biomechanics ───────────────────────────────────────────────────────────

fn bench_center_of_mass(c: &mut Criterion) {
    let masses: Vec<f32> = (0..100).map(|i| 1.0 + i as f32 * 0.1).collect();
    let positions: Vec<Vec3> = (0..100)
        .map(|i| Vec3::new(i as f32 * 0.1, i as f32 * 0.05, 0.0))
        .collect();
    c.bench_function("center_of_mass_100", |b| {
        b.iter(|| black_box(biomechanics::center_of_mass(&masses, &positions)))
    });
}

fn bench_grf(c: &mut Criterion) {
    c.bench_function("ground_reaction_force", |b| {
        b.iter(|| {
            black_box(biomechanics::ground_reaction_force(
                black_box(70.0),
                black_box(9.81),
                black_box(0.6),
            ))
        })
    });
}

fn bench_support_polygon(c: &mut Criterion) {
    let contacts = vec![
        Vec3::new(-0.1, 0.0, -0.2),
        Vec3::new(0.1, 0.0, -0.2),
        Vec3::new(0.1, 0.0, 0.2),
        Vec3::new(-0.1, 0.0, 0.2),
    ];
    c.bench_function("support_polygon_4", |b| {
        b.iter(|| black_box(biomechanics::support_polygon(black_box(&contacts))))
    });
}

fn bench_zmp(c: &mut Criterion) {
    c.bench_function("zero_moment_point", |b| {
        b.iter(|| {
            black_box(biomechanics::zero_moment_point(
                black_box(Vec3::new(0.5, 1.0, 0.3)),
                black_box(Vec3::new(0.5, 0.0, 0.0)),
                black_box(9.81),
            ))
        })
    });
}

fn bench_stability_margin(c: &mut Criterion) {
    use hisab::Vec2;
    let polygon = vec![
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, 1.0),
    ];
    c.bench_function("stability_margin", |b| {
        b.iter(|| {
            black_box(biomechanics::stability_margin(
                black_box(Vec2::new(0.3, 0.2)),
                black_box(&polygon),
            ))
        })
    });
}

fn bench_balance_margin(c: &mut Criterion) {
    c.bench_function("balance_margin_1d", |b| {
        b.iter(|| {
            black_box(biomechanics::balance_margin(
                black_box(0.5),
                black_box(0.0),
                black_box(1.0),
            ))
        })
    });
}

// ── Gait ───────────────────────────────────────────────────────────────────

fn bench_gait_limb_phase(c: &mut Criterion) {
    let gait = Gait::human_walk();
    c.bench_function("gait_limb_phase", |b| {
        b.iter(|| black_box(gait.limb_phase(black_box(0), black_box(0.75))))
    });
}

fn bench_gait_speed(c: &mut Criterion) {
    let gait = Gait::human_walk();
    c.bench_function("gait_speed", |b| b.iter(|| black_box(gait.speed())));
}

fn bench_gait_blend(c: &mut Criterion) {
    let walk = Gait::human_walk();
    let run = Gait::human_run();
    c.bench_function("gait_blend", |b| {
        b.iter(|| {
            black_box(Gait::blend(
                black_box(&walk),
                black_box(&run),
                black_box(0.5),
            ))
        })
    });
}

fn bench_gait_controller_update(c: &mut Criterion) {
    let mut controller = GaitController::bipedal_default();
    controller.set_speed(3.0);
    c.bench_function("gait_controller_update", |b| {
        b.iter(|| {
            controller.update(black_box(0.016));
        })
    });
}

fn bench_foot_placements(c: &mut Criterion) {
    let gait = Gait::quadruped_trot();
    c.bench_function("foot_placements_4_limbs", |b| {
        b.iter(|| {
            black_box(gait.foot_placements(
                black_box(0.5),
                black_box(Vec3::ZERO),
                black_box(Vec3::X),
            ))
        })
    });
}

// ── Kinematics ─────────────────────────────────────────────────────────────

fn bench_fk_20(c: &mut Criterion) {
    let skeleton = build_skeleton(20);
    let pose = Pose::rest(20);
    c.bench_function("fk_biped_20", |b| {
        b.iter(|| {
            black_box(forward_kinematics(
                black_box(&skeleton),
                black_box(&pose),
                black_box(Vec3::ZERO),
                black_box(Quat::IDENTITY),
            ))
        })
    });
}

fn bench_fk_100(c: &mut Criterion) {
    let skeleton = build_skeleton(100);
    let pose = Pose::rest(100);
    c.bench_function("fk_chain_100", |b| {
        b.iter(|| {
            black_box(forward_kinematics(
                black_box(&skeleton),
                black_box(&pose),
                black_box(Vec3::ZERO),
                black_box(Quat::IDENTITY),
            ))
        })
    });
}

// ── Pose ───────────────────────────────────────────────────────────────────

fn bench_pose_blend(c: &mut Criterion) {
    let mut a = Pose::rest(20);
    let mut b = Pose::rest(20);
    for i in 0..20 {
        a.set_joint(BoneId(i), Quat::from_rotation_z(0.1 * i as f32));
        b.set_joint(BoneId(i), Quat::from_rotation_x(0.2 * i as f32));
    }
    c.bench_function("pose_blend_20", |b_| {
        b_.iter(|| black_box(Pose::blend(black_box(&a), black_box(&b), black_box(0.5))))
    });
}

// ── Body ───────────────────────────────────────────────────────────────────

fn bench_body_update(c: &mut Criterion) {
    let mut body = Body::new(build_skeleton(20));
    c.bench_function("body_update_20", |b| {
        b.iter(|| {
            body.update(black_box(Vec3::ZERO), black_box(Quat::IDENTITY));
        })
    });
}

// ── IK ─────────────────────────────────────────────────────────────────────

fn bench_ik_two_bone(c: &mut Criterion) {
    let (skeleton, chain) = build_two_bone_chain();
    let target = IKTarget {
        position: Vec3::new(0.2, -0.4, 0.0),
        orientation: None,
        pole_vector: Some(Vec3::Z),
    };
    c.bench_function("ik_two_bone", |b| {
        b.iter(|| {
            black_box(solve_two_bone(
                black_box(&chain),
                black_box(&target),
                black_box(&skeleton),
                black_box(Vec3::ZERO),
                black_box(Quat::IDENTITY),
            ))
        })
    });
}

fn bench_ik_fabrik_5(c: &mut Criterion) {
    let skeleton = build_skeleton(5);
    let chain = IKChain::new(
        (0..5).map(BoneId).collect(),
        (0..4)
            .map(|i| Joint::human_knee(BoneId(i), BoneId(i + 1)))
            .collect(),
    );
    let target = IKTarget {
        position: Vec3::new(0.1, 0.3, 0.0),
        orientation: None,
        pole_vector: None,
    };
    c.bench_function("ik_fabrik_5_bone", |b| {
        b.iter(|| {
            black_box(solve_fabrik(
                black_box(&chain),
                black_box(&target),
                black_box(&skeleton),
                black_box(Vec3::ZERO),
                black_box(Quat::IDENTITY),
                black_box(10),
                black_box(0.001),
            ))
        })
    });
}

// ── Fatigue ────────────────────────────────────────────────────────────────

fn bench_fatigue_update(c: &mut Criterion) {
    let mut state = FatigueState::fresh();
    c.bench_function("fatigue_update", |b| {
        b.iter(|| {
            state.update(black_box(0.8), black_box(0.001));
        })
    });
}

// ── Allometry ──────────────────────────────────────────────────────────────

fn bench_allometric_skeleton(c: &mut Criterion) {
    let params = AllometricParams::mammalian();
    c.bench_function("allometric_skeleton_biped_70kg", |b| {
        b.iter(|| {
            black_box(allometric_skeleton(
                black_box(70.0),
                black_box(BodyPlan::Bipedal),
                black_box(&params),
            ))
        })
    });
}

// ── Morphology ─────────────────────────────────────────────────────────────

fn bench_apply_morphology(c: &mut Criterion) {
    let skeleton = build_skeleton(20);
    let morph = Morphology::heavy();
    c.bench_function("apply_morphology_20", |b| {
        b.iter(|| black_box(apply_morphology(black_box(&skeleton), black_box(&morph))))
    });
}

// ── Groups ─────────────────────────────────────────────────────────────────

criterion_group!(
    skeleton,
    bench_skeleton_total_mass,
    bench_skeleton_find_bone,
    bench_skeleton_chain_to_root,
    bench_skeleton_children,
);

criterion_group!(
    muscle,
    bench_muscle_force,
    bench_muscle_force_at,
    bench_muscle_activation_update,
);

criterion_group!(
    biomech,
    bench_center_of_mass,
    bench_grf,
    bench_support_polygon,
    bench_zmp,
    bench_stability_margin,
    bench_balance_margin,
);

criterion_group!(
    gaits,
    bench_gait_limb_phase,
    bench_gait_speed,
    bench_gait_blend,
    bench_gait_controller_update,
    bench_foot_placements,
);

criterion_group!(
    kinematics,
    bench_fk_20,
    bench_fk_100,
    bench_pose_blend,
    bench_body_update,
);

criterion_group!(ik, bench_ik_two_bone, bench_ik_fabrik_5,);

criterion_group!(
    dynamics,
    bench_fatigue_update,
    bench_allometric_skeleton,
    bench_apply_morphology,
);

criterion_main!(skeleton, muscle, biomech, gaits, kinematics, ik, dynamics);
