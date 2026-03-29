use sharira::biomechanics;
use sharira::{Bone, BoneId, Gait, Joint, Muscle, MuscleGroup, Skeleton};

fn main() {
    // Build a minimal biped skeleton
    let mut skeleton = Skeleton::new("human");
    skeleton.add_bone(Bone::new(BoneId(0), "pelvis", 0.2, 10.0, None));
    skeleton.add_bone(
        Bone::new(BoneId(1), "femur_l", 0.45, 8.0, Some(BoneId(0)))
            .with_position([-0.1, -0.1, 0.0]),
    );
    skeleton.add_bone(
        Bone::new(BoneId(2), "tibia_l", 0.4, 5.0, Some(BoneId(1))).with_position([0.0, -0.45, 0.0]),
    );

    println!(
        "Skeleton: {} ({} bones, {:.1} kg)",
        skeleton.name,
        skeleton.bone_count(),
        skeleton.total_mass()
    );

    // Attach a joint
    let knee = Joint::human_knee(BoneId(1), BoneId(2));
    println!(
        "Knee: {:?}, {} DOF",
        knee.joint_type,
        knee.joint_type.degrees_of_freedom()
    );

    // Attach a muscle with full Hill model
    let mut quad = Muscle::new(
        "quadriceps",
        BoneId(1),
        BoneId(2),
        MuscleGroup::Extensor,
        5000.0,
        0.3,
    );
    quad.set_activation(0.8);
    let force = quad.current_force(0.3);
    println!("Quadriceps force at rest length: {:.0} N", force);

    // Force-velocity: shortening reduces force
    let shortening_force = quad.force_at(0.3, -3.0);
    println!(
        "Quadriceps force while shortening: {:.0} N",
        shortening_force
    );

    // Walk gait
    let walk = Gait::human_walk();
    println!("Walk speed: {:.2} m/s", walk.speed());
    println!("Limb phase at t=0.0: {:?}", walk.limb_phase(0, 0.0));
    println!("Limb phase at t=0.8: {:?}", walk.limb_phase(0, 0.8));

    // Center of mass
    let masses: Vec<f32> = skeleton.bones().iter().map(|b| b.mass).collect();
    let positions: Vec<[f32; 3]> = skeleton.bones().iter().map(|b| b.local_position).collect();
    let com = biomechanics::center_of_mass(&masses, &positions);
    println!(
        "Center of mass: [{:.3}, {:.3}, {:.3}]",
        com[0], com[1], com[2]
    );
}
