use serde::{Deserialize, Serialize};

/// Unique bone identifier within a skeleton.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoneId(pub u16);

/// A bone in a skeleton hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bone {
    pub id: BoneId,
    pub name: String,
    pub parent: Option<BoneId>,
    pub length: f32,            // meters
    pub mass: f32,              // kg
    pub local_position: [f32; 3],  // offset from parent joint
    pub local_rotation: [f32; 4],  // quaternion [x,y,z,w]
}

/// A complete skeleton (bone hierarchy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<Bone>,
}

impl Skeleton {
    /// Find a bone by name.
    #[must_use]
    pub fn find_bone(&self, name: &str) -> Option<&Bone> {
        self.bones.iter().find(|b| b.name == name)
    }

    /// Find a bone by ID.
    #[must_use]
    pub fn get_bone(&self, id: BoneId) -> Option<&Bone> {
        self.bones.iter().find(|b| b.id == id)
    }

    /// Total mass of all bones.
    #[must_use]
    pub fn total_mass(&self) -> f32 {
        self.bones.iter().map(|b| b.mass).sum()
    }

    /// Number of bones.
    #[must_use]
    #[inline]
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// Get all root bones (no parent).
    #[must_use]
    pub fn roots(&self) -> Vec<&Bone> {
        self.bones.iter().filter(|b| b.parent.is_none()).collect()
    }

    /// Get children of a bone.
    #[must_use]
    pub fn children(&self, parent_id: BoneId) -> Vec<&Bone> {
        self.bones.iter().filter(|b| b.parent == Some(parent_id)).collect()
    }

    /// Chain from bone to root (inclusive).
    #[must_use]
    pub fn chain_to_root(&self, bone_id: BoneId) -> Vec<BoneId> {
        let mut chain = vec![bone_id];
        let mut current = bone_id;
        while let Some(bone) = self.get_bone(current) {
            if let Some(parent) = bone.parent {
                chain.push(parent);
                current = parent;
            } else {
                break;
            }
        }
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_skeleton() -> Skeleton {
        Skeleton {
            name: "test".into(),
            bones: vec![
                Bone { id: BoneId(0), name: "root".into(), parent: None, length: 0.5, mass: 10.0, local_position: [0.0; 3], local_rotation: [0.0, 0.0, 0.0, 1.0] },
                Bone { id: BoneId(1), name: "spine".into(), parent: Some(BoneId(0)), length: 0.4, mass: 8.0, local_position: [0.0, 0.5, 0.0], local_rotation: [0.0, 0.0, 0.0, 1.0] },
                Bone { id: BoneId(2), name: "head".into(), parent: Some(BoneId(1)), length: 0.2, mass: 5.0, local_position: [0.0, 0.4, 0.0], local_rotation: [0.0, 0.0, 0.0, 1.0] },
                Bone { id: BoneId(3), name: "left_arm".into(), parent: Some(BoneId(1)), length: 0.6, mass: 4.0, local_position: [-0.2, 0.3, 0.0], local_rotation: [0.0, 0.0, 0.0, 1.0] },
                Bone { id: BoneId(4), name: "right_arm".into(), parent: Some(BoneId(1)), length: 0.6, mass: 4.0, local_position: [0.2, 0.3, 0.0], local_rotation: [0.0, 0.0, 0.0, 1.0] },
            ],
        }
    }

    #[test]
    fn find_bone_by_name() {
        let s = test_skeleton();
        assert!(s.find_bone("spine").is_some());
        assert!(s.find_bone("tail").is_none());
    }

    #[test]
    fn total_mass() {
        let s = test_skeleton();
        assert!((s.total_mass() - 31.0).abs() < 0.01);
    }

    #[test]
    fn bone_count() {
        assert_eq!(test_skeleton().bone_count(), 5);
    }

    #[test]
    fn single_root() {
        let s = test_skeleton();
        assert_eq!(s.roots().len(), 1);
        assert_eq!(s.roots()[0].name, "root");
    }

    #[test]
    fn spine_has_children() {
        let s = test_skeleton();
        let children = s.children(BoneId(1));
        assert_eq!(children.len(), 3); // head, left_arm, right_arm
    }

    #[test]
    fn chain_to_root() {
        let s = test_skeleton();
        let chain = s.chain_to_root(BoneId(2)); // head → spine → root
        assert_eq!(chain, vec![BoneId(2), BoneId(1), BoneId(0)]);
    }

    #[test]
    fn root_chain_is_self() {
        let s = test_skeleton();
        let chain = s.chain_to_root(BoneId(0));
        assert_eq!(chain, vec![BoneId(0)]);
    }
}
