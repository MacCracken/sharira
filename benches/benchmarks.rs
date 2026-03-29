use criterion::{Criterion, black_box, criterion_group, criterion_main};
use sharira::biomechanics;
use sharira::{Bone, BoneId, Gait, Muscle, MuscleGroup, Skeleton};

fn build_skeleton(bone_count: u16) -> Skeleton {
    let mut skeleton = Skeleton::new("bench");
    skeleton.add_bone(Bone::new(BoneId(0), "root", 0.2, 5.0, None));
    for i in 1..bone_count {
        skeleton.add_bone(Bone::new(
            BoneId(i),
            format!("bone_{i}"),
            0.1,
            1.0,
            Some(BoneId(i - 1)),
        ));
    }
    skeleton
}

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

fn bench_center_of_mass(c: &mut Criterion) {
    let masses: Vec<f32> = (0..100).map(|i| 1.0 + i as f32 * 0.1).collect();
    let positions: Vec<[f32; 3]> = (0..100)
        .map(|i| [i as f32 * 0.1, i as f32 * 0.05, 0.0])
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

fn bench_balance_margin(c: &mut Criterion) {
    c.bench_function("balance_margin", |b| {
        b.iter(|| {
            black_box(biomechanics::balance_margin(
                black_box(0.5),
                black_box(0.0),
                black_box(1.0),
            ))
        })
    });
}

criterion_group!(
    benches,
    bench_skeleton_total_mass,
    bench_skeleton_find_bone,
    bench_skeleton_chain_to_root,
    bench_skeleton_children,
    bench_muscle_force,
    bench_center_of_mass,
    bench_grf,
    bench_gait_limb_phase,
    bench_gait_speed,
    bench_balance_margin,
);
criterion_main!(benches);
