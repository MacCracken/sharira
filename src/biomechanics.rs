use hisab::{Vec2, Vec3};

/// Center of mass from weighted positions.
///
/// CoM = Σ(m_i × p_i) / Σ(m_i)
#[must_use]
pub fn center_of_mass(masses: &[f32], positions: &[Vec3]) -> Vec3 {
    if masses.len() != positions.len() || masses.is_empty() {
        return Vec3::ZERO;
    }
    let total_mass: f32 = masses.iter().sum();
    if total_mass <= 0.0 {
        return Vec3::ZERO;
    }

    let mut com = Vec3::ZERO;
    for (m, p) in masses.iter().zip(positions.iter()) {
        com += *m * *p;
    }
    com / total_mass
}

/// Ground reaction force for bipedal stance (simplified).
///
/// GRF = mass × g during stance. During swing, GRF = 0.
#[must_use]
#[inline]
pub fn ground_reaction_force(mass_kg: f32, gravity: f32, duty_factor: f32) -> f32 {
    if duty_factor <= 0.0 {
        return 0.0;
    }
    mass_kg * gravity / duty_factor
}

/// Metabolic cost of locomotion (cost of transport).
///
/// CoT = energy_per_step / (mass × distance)
#[must_use]
#[inline]
pub fn cost_of_transport(energy_j: f32, mass_kg: f32, distance_m: f32) -> f32 {
    if mass_kg <= 0.0 || distance_m <= 0.0 {
        return 0.0;
    }
    energy_j / (mass_kg * distance_m)
}

/// Balance margin (1D) — distance from CoM projection to support polygon edge.
/// Positive = stable, negative = falling.
#[must_use]
pub fn balance_margin(com_x: f32, support_min_x: f32, support_max_x: f32) -> f32 {
    let to_min = com_x - support_min_x;
    let to_max = support_max_x - com_x;
    to_min.min(to_max)
}

// ---------------------------------------------------------------------------
// 2D Support Polygon & Stability
// ---------------------------------------------------------------------------

/// Compute the 2D support polygon from ground contact points.
///
/// Projects 3D contact points onto the XZ ground plane and computes
/// the convex hull. Returns vertices in counter-clockwise order.
#[must_use]
pub fn support_polygon(contact_points: &[Vec3]) -> Vec<Vec2> {
    if contact_points.is_empty() {
        return Vec::new();
    }
    let points_2d: Vec<Vec2> = contact_points.iter().map(|p| Vec2::new(p.x, p.z)).collect();
    hisab::geo::convex_hull_2d(&points_2d)
}

/// Compute the Zero Moment Point (ZMP).
///
/// The ZMP is the point on the ground plane where the net horizontal moment is zero.
/// For a body with CoM at `com`, experiencing `com_acceleration` under `gravity`:
///
/// `ZMP_x = CoM_x - CoM_y * accel_x / (gravity + accel_y)`
/// `ZMP_z = CoM_z - CoM_y * accel_z / (gravity + accel_y)`
///
/// Returns `None` if the denominator is zero (free fall).
/// Uses Y-up convention: gravity acts in -Y direction.
#[must_use]
pub fn zero_moment_point(com: Vec3, com_acceleration: Vec3, gravity: f32) -> Option<Vec2> {
    let denom = gravity + com_acceleration.y;
    if denom.abs() < 1e-8 {
        return None; // free fall
    }
    let zmp_x = com.x - com.y * com_acceleration.x / denom;
    let zmp_z = com.z - com.y * com_acceleration.z / denom;
    Some(Vec2::new(zmp_x, zmp_z))
}

/// Stability margin — minimum distance from a point to the nearest edge of a convex polygon.
///
/// Positive = point inside polygon (stable).
/// Negative = point outside polygon (falling).
/// Zero = point on the edge.
///
/// `point`: 2D point to test (e.g., ZMP or CoM projection)
/// `polygon`: convex polygon vertices in counter-clockwise order
#[must_use]
pub fn stability_margin(point: Vec2, polygon: &[Vec2]) -> f32 {
    if polygon.is_empty() {
        return f32::NEG_INFINITY;
    }
    if polygon.len() == 1 {
        return -(point - polygon[0]).length();
    }
    if polygon.len() == 2 {
        // Line segment: distance to segment, always negative (not a polygon)
        return -point_to_segment_distance(point, polygon[0], polygon[1]);
    }

    // Determine winding order by computing signed area
    let signed_area = polygon_signed_area(polygon);
    // For each edge, compute signed distance (positive = inside)
    // Normal direction depends on winding: CCW uses left normal, CW uses right normal
    let mut min_distance = f32::INFINITY;
    let n = polygon.len();
    for i in 0..n {
        let a = polygon[i];
        let b = polygon[(i + 1) % n];
        let edge = b - a;
        // Inward normal: for CCW, rotate edge 90° CW = (edge.y, -edge.x)
        // For CW winding, flip the normal
        let normal = if signed_area >= 0.0 {
            Vec2::new(-edge.y, edge.x) // CCW: left normal is inward
        } else {
            Vec2::new(edge.y, -edge.x) // CW: right normal is inward
        };
        let normal_len = normal.length();
        if normal_len < 1e-10 {
            continue;
        }
        let signed_dist = (point - a).dot(normal) / normal_len;
        min_distance = min_distance.min(signed_dist);
    }
    min_distance
}

/// Signed area of a polygon (positive = CCW, negative = CW).
#[must_use]
#[inline]
fn polygon_signed_area(polygon: &[Vec2]) -> f32 {
    let n = polygon.len();
    let mut area = 0.0_f32;
    for i in 0..n {
        let a = polygon[i];
        let b = polygon[(i + 1) % n];
        area += a.x * b.y - b.x * a.y;
    }
    area * 0.5
}

/// Distance from point to line segment.
#[must_use]
#[inline]
fn point_to_segment_distance(point: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ab_sq = ab.dot(ab);
    if ab_sq < 1e-10 {
        return (point - a).length();
    }
    let t = ((point - a).dot(ab) / ab_sq).clamp(0.0, 1.0);
    let closest = a + t * ab;
    (point - closest).length()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn com_two_equal_masses() {
        let com = center_of_mass(
            &[1.0, 1.0],
            &[Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0)],
        );
        assert!((com.x - 1.0).abs() < 0.01);
    }

    #[test]
    fn com_weighted() {
        let com = center_of_mass(
            &[3.0, 1.0],
            &[Vec3::new(0.0, 0.0, 0.0), Vec3::new(4.0, 0.0, 0.0)],
        );
        assert!(
            (com.x - 1.0).abs() < 0.01,
            "CoM should be at 1.0, got {}",
            com.x
        );
    }

    #[test]
    fn grf_walking() {
        let grf = ground_reaction_force(70.0, 9.81, 0.6);
        assert!(
            grf > 70.0 * 9.81,
            "GRF should exceed body weight during walking"
        );
    }

    #[test]
    fn cot_walking() {
        let cot = cost_of_transport(240.0, 70.0, 1.0);
        assert!(
            (cot - 3.43).abs() < 0.1,
            "walking CoT should be ~3.4, got {cot}"
        );
    }

    #[test]
    fn balance_stable() {
        let margin = balance_margin(0.5, 0.0, 1.0);
        assert!(margin > 0.0, "CoM within support → positive margin");
    }

    #[test]
    fn balance_unstable() {
        let margin = balance_margin(1.5, 0.0, 1.0);
        assert!(margin < 0.0, "CoM outside support → negative margin");
    }

    #[test]
    fn com_empty_returns_zero() {
        let com = center_of_mass(&[], &[]);
        assert_eq!(com, Vec3::ZERO);
    }

    // Support polygon tests

    #[test]
    fn support_polygon_empty() {
        let poly = support_polygon(&[]);
        assert!(poly.is_empty());
    }

    #[test]
    fn support_polygon_quad() {
        // 4 foot contact points forming a rectangle on the ground (Y=0)
        let contacts = [
            Vec3::new(-0.1, 0.0, -0.2),
            Vec3::new(0.1, 0.0, -0.2),
            Vec3::new(0.1, 0.0, 0.2),
            Vec3::new(-0.1, 0.0, 0.2),
        ];
        let poly = support_polygon(&contacts);
        assert!(poly.len() >= 4, "rectangle should have 4 hull vertices");
    }

    #[test]
    fn support_polygon_with_interior_points() {
        // Points including interior points that should be excluded from hull
        let contacts = [
            Vec3::new(-1.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(-1.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0), // interior
        ];
        let poly = support_polygon(&contacts);
        assert_eq!(poly.len(), 4, "interior point should not be in hull");
    }

    // ZMP tests

    #[test]
    fn zmp_stationary() {
        // Stationary body: acceleration = 0, ZMP = CoM projection
        let com = Vec3::new(0.5, 1.0, 0.3);
        let zmp = zero_moment_point(com, Vec3::ZERO, 9.81).unwrap();
        assert!(
            (zmp.x - 0.5).abs() < 1e-5,
            "ZMP x should equal CoM x for stationary body"
        );
        assert!(
            (zmp.y - 0.3).abs() < 1e-5,
            "ZMP y should equal CoM z for stationary body"
        );
    }

    #[test]
    fn zmp_with_acceleration() {
        // Body accelerating forward (positive X)
        let com = Vec3::new(0.0, 1.0, 0.0);
        let accel = Vec3::new(2.0, 0.0, 0.0);
        let zmp = zero_moment_point(com, accel, 9.81).unwrap();
        // ZMP_x = 0 - 1.0 * 2.0 / 9.81 ≈ -0.204
        assert!(
            zmp.x < 0.0,
            "ZMP should shift backwards when accelerating forward"
        );
    }

    #[test]
    fn zmp_freefall_returns_none() {
        let com = Vec3::new(0.0, 1.0, 0.0);
        let accel = Vec3::new(0.0, -9.81, 0.0); // gravity + accel_y = 0
        let zmp = zero_moment_point(com, accel, 9.81);
        assert!(zmp.is_none(), "ZMP undefined in free fall");
    }

    // Stability margin tests

    #[test]
    fn stability_margin_centered() {
        // Square support polygon, point at center
        let polygon = vec![
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
        ];
        let margin = stability_margin(Vec2::ZERO, &polygon);
        assert!(
            (margin - 1.0).abs() < 0.01,
            "center of unit square should have margin 1.0, got {margin}"
        );
    }

    #[test]
    fn stability_margin_on_edge() {
        let polygon = vec![
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
        ];
        let margin = stability_margin(Vec2::new(1.0, 0.0), &polygon);
        assert!(
            margin.abs() < 0.01,
            "point on edge should have margin ~0, got {margin}"
        );
    }

    #[test]
    fn stability_margin_outside() {
        let polygon = vec![
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
        ];
        let margin = stability_margin(Vec2::new(2.0, 0.0), &polygon);
        assert!(
            margin < 0.0,
            "point outside should have negative margin, got {margin}"
        );
    }

    #[test]
    fn stability_margin_empty_polygon() {
        let margin = stability_margin(Vec2::ZERO, &[]);
        assert!(margin == f32::NEG_INFINITY);
    }
}
