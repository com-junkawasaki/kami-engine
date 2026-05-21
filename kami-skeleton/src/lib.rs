//! kami-skeleton: Skeletal animation (bone hierarchy, skinning, blend).
//!
//! glTF-compatible bone system. GPU skinning via joint matrices.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// A single bone in the skeleton.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub local_position: [f32; 3],
    pub local_rotation: [f32; 4], // quaternion xyzw
    pub local_scale: [f32; 3],
    pub inverse_bind: [[f32; 4]; 4], // inverse bind matrix (column-major)
}

/// Skeleton: bone hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
}

/// Animation keyframe.
#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32,
    pub position: Option<Vec3>,
    pub rotation: Option<Quat>,
    pub scale: Option<Vec3>,
}

/// Animation clip for one bone.
#[derive(Debug, Clone)]
pub struct BoneTrack {
    pub bone_index: usize,
    pub keyframes: Vec<Keyframe>,
}

/// Animation clip.
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<BoneTrack>,
    pub looping: bool,
}

/// Joint matrix for GPU skinning (4x4, column-major).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct JointMatrix {
    pub mat: [[f32; 4]; 4],
}

impl Skeleton {
    /// Compute world transforms for all bones at a given animation time.
    pub fn evaluate(&self, clip: &AnimationClip, time: f32) -> Vec<Mat4> {
        let n = self.bones.len();
        let mut local_transforms = Vec::with_capacity(n);

        for (i, bone) in self.bones.iter().enumerate() {
            let pos = Vec3::from(bone.local_position);
            let rot = Quat::from_array(bone.local_rotation);
            let scl = Vec3::from(bone.local_scale);

            // Find track for this bone and interpolate
            let (p, r, s) = if let Some(track) = clip.tracks.iter().find(|t| t.bone_index == i) {
                interpolate_track(track, time)
            } else {
                (pos, rot, scl)
            };

            local_transforms.push(Mat4::from_scale_rotation_translation(s, r, p));
        }

        // Compute world transforms (parent-first order assumed)
        let mut world = vec![Mat4::IDENTITY; n];
        for i in 0..n {
            world[i] = match self.bones[i].parent {
                Some(p) => world[p] * local_transforms[i],
                None => local_transforms[i],
            };
        }
        world
    }

    /// Compute world transforms with anatomical joint constraints applied.
    ///
    /// Each entry in `constraints` maps a bone index to its constraint.
    /// Bones without constraints are unclamped.
    pub fn evaluate_constrained(
        &self,
        clip: &AnimationClip,
        time: f32,
        constraints: &[(usize, JointConstraint)],
    ) -> Vec<Mat4> {
        let n = self.bones.len();
        let mut local_transforms = Vec::with_capacity(n);

        for (i, bone) in self.bones.iter().enumerate() {
            let pos = Vec3::from(bone.local_position);
            let rot = Quat::from_array(bone.local_rotation);
            let scl = Vec3::from(bone.local_scale);

            let (p, mut r, s) =
                if let Some(track) = clip.tracks.iter().find(|t| t.bone_index == i) {
                    interpolate_track(track, time)
                } else {
                    (pos, rot, scl)
                };

            // Apply joint constraint if present for this bone
            if let Some((_, constraint)) = constraints.iter().find(|(idx, _)| *idx == i) {
                r = constraint.clamp(r);
            }

            local_transforms.push(Mat4::from_scale_rotation_translation(s, r, p));
        }

        let mut world = vec![Mat4::IDENTITY; n];
        for i in 0..n {
            world[i] = match self.bones[i].parent {
                Some(p) => world[p] * local_transforms[i],
                None => local_transforms[i],
            };
        }
        world
    }

    /// Compute joint matrices with anatomical constraints for GPU upload.
    pub fn joint_matrices_constrained(
        &self,
        clip: &AnimationClip,
        time: f32,
        constraints: &[(usize, JointConstraint)],
    ) -> Vec<JointMatrix> {
        let world = self.evaluate_constrained(clip, time, constraints);
        self.bones
            .iter()
            .enumerate()
            .map(|(i, bone)| {
                let inv_bind = Mat4::from_cols_array_2d(&bone.inverse_bind);
                let joint = world[i] * inv_bind;
                JointMatrix {
                    mat: joint.to_cols_array_2d(),
                }
            })
            .collect()
    }

    /// Build constraint index from bone names using `default_humanoid_constraints`.
    ///
    /// Returns pairs of `(bone_index, JointConstraint)` for bones found in this
    /// skeleton by name.
    pub fn build_humanoid_constraints(&self) -> Vec<(usize, JointConstraint)> {
        let defaults = default_humanoid_constraints();
        let mut result = Vec::new();
        for (name, constraint) in defaults {
            if let Some(idx) = self.bones.iter().position(|b| b.name == name) {
                result.push((idx, constraint));
            }
        }
        result
    }

    /// Compute joint matrices for GPU upload (world * inverse_bind).
    pub fn joint_matrices(&self, clip: &AnimationClip, time: f32) -> Vec<JointMatrix> {
        let world = self.evaluate(clip, time);
        self.bones
            .iter()
            .enumerate()
            .map(|(i, bone)| {
                let inv_bind = Mat4::from_cols_array_2d(&bone.inverse_bind);
                let joint = world[i] * inv_bind;
                JointMatrix {
                    mat: joint.to_cols_array_2d(),
                }
            })
            .collect()
    }
}

/// Anatomical joint rotation constraint (Euler angles in radians).
///
/// Constrains bone rotation to prevent humanly impossible poses.
/// Each axis specifies `[min, max]` in radians. Applied after interpolation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointConstraint {
    /// Minimum Euler angle per axis `[x, y, z]` in radians.
    pub min: [f32; 3],
    /// Maximum Euler angle per axis `[x, y, z]` in radians.
    pub max: [f32; 3],
}

impl JointConstraint {
    /// Clamp a quaternion rotation to the Euler angle limits.
    ///
    /// Decomposes the quaternion to Euler XYZ, clamps each axis, then
    /// recomposes. Suitable for humanoid bones where gimbal lock is
    /// unlikely within normal anatomical ranges.
    pub fn clamp(&self, rotation: Quat) -> Quat {
        let (x, y, z) = quat_to_euler_xyz(rotation);
        let cx = x.clamp(self.min[0], self.max[0]);
        let cy = y.clamp(self.min[1], self.max[1]);
        let cz = z.clamp(self.min[2], self.max[2]);
        euler_xyz_to_quat(cx, cy, cz)
    }
}

/// Default anatomical constraints for VRM humanoid bones.
///
/// Returns `(bone_name, JointConstraint)` pairs covering standard humanoid
/// skeleton bones. Values derived from orthopedic range-of-motion references.
pub fn default_humanoid_constraints() -> Vec<(&'static str, JointConstraint)> {
    let d = std::f32::consts::PI / 180.0;
    vec![
        ("head", JointConstraint { min: [-60.0 * d, -80.0 * d, -40.0 * d], max: [60.0 * d, 80.0 * d, 40.0 * d] }),
        ("neck", JointConstraint { min: [-30.0 * d, -45.0 * d, -30.0 * d], max: [30.0 * d, 45.0 * d, 30.0 * d] }),
        ("spine", JointConstraint { min: [-30.0 * d, -30.0 * d, -20.0 * d], max: [30.0 * d, 30.0 * d, 20.0 * d] }),
        ("chest", JointConstraint { min: [-15.0 * d, -15.0 * d, -10.0 * d], max: [15.0 * d, 15.0 * d, 10.0 * d] }),
        ("hips", JointConstraint { min: [-30.0 * d, -30.0 * d, -15.0 * d], max: [30.0 * d, 30.0 * d, 15.0 * d] }),
        ("leftUpperArm", JointConstraint { min: [-60.0 * d, -45.0 * d, -30.0 * d], max: [90.0 * d, 90.0 * d, 180.0 * d] }),
        ("rightUpperArm", JointConstraint { min: [-60.0 * d, -90.0 * d, -180.0 * d], max: [90.0 * d, 45.0 * d, 30.0 * d] }),
        ("leftLowerArm", JointConstraint { min: [-5.0 * d, 0.0, -5.0 * d], max: [5.0 * d, 145.0 * d, 5.0 * d] }),
        ("rightLowerArm", JointConstraint { min: [-5.0 * d, -145.0 * d, -5.0 * d], max: [5.0 * d, 0.0, 5.0 * d] }),
        ("leftUpperLeg", JointConstraint { min: [-30.0 * d, -45.0 * d, -20.0 * d], max: [120.0 * d, 30.0 * d, 45.0 * d] }),
        ("rightUpperLeg", JointConstraint { min: [-30.0 * d, -30.0 * d, -45.0 * d], max: [120.0 * d, 45.0 * d, 20.0 * d] }),
        ("leftLowerLeg", JointConstraint { min: [-140.0 * d, -5.0 * d, -5.0 * d], max: [0.0, 5.0 * d, 5.0 * d] }),
        ("rightLowerLeg", JointConstraint { min: [-140.0 * d, -5.0 * d, -5.0 * d], max: [0.0, 5.0 * d, 5.0 * d] }),
    ]
}

/// Decompose quaternion to intrinsic Euler XYZ angles.
fn quat_to_euler_xyz(q: Quat) -> (f32, f32, f32) {
    let (x, y, z, w) = (q.x, q.y, q.z, q.w);
    let sinr_cosp = 2.0 * (w * x + y * z);
    let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
    let roll = sinr_cosp.atan2(cosr_cosp);
    let sinp = 2.0 * (w * y - z * x);
    let pitch = if sinp.abs() >= 1.0 {
        std::f32::consts::FRAC_PI_2.copysign(sinp)
    } else {
        sinp.asin()
    };
    let siny_cosp = 2.0 * (w * z + x * y);
    let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
    let yaw = siny_cosp.atan2(cosy_cosp);
    (roll, pitch, yaw)
}

/// Compose intrinsic Euler XYZ angles to quaternion.
fn euler_xyz_to_quat(x: f32, y: f32, z: f32) -> Quat {
    Quat::from_rotation_z(z) * Quat::from_rotation_y(y) * Quat::from_rotation_x(x)
}

fn interpolate_track(track: &BoneTrack, time: f32) -> (Vec3, Quat, Vec3) {
    let kfs = &track.keyframes;
    if kfs.is_empty() {
        return (Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
    }
    if kfs.len() == 1 || time <= kfs[0].time {
        let k = &kfs[0];
        return (
            k.position.unwrap_or(Vec3::ZERO),
            k.rotation.unwrap_or(Quat::IDENTITY),
            k.scale.unwrap_or(Vec3::ONE),
        );
    }
    let last = &kfs[kfs.len() - 1];
    if time >= last.time {
        return (
            last.position.unwrap_or(Vec3::ZERO),
            last.rotation.unwrap_or(Quat::IDENTITY),
            last.scale.unwrap_or(Vec3::ONE),
        );
    }

    // Find bracket
    let mut i = 0;
    while i < kfs.len() - 1 && kfs[i + 1].time < time {
        i += 1;
    }
    let a = &kfs[i];
    let b = &kfs[i + 1];
    let t = (time - a.time) / (b.time - a.time);

    let pos = a
        .position
        .unwrap_or(Vec3::ZERO)
        .lerp(b.position.unwrap_or(Vec3::ZERO), t);
    let rot = a
        .rotation
        .unwrap_or(Quat::IDENTITY)
        .slerp(b.rotation.unwrap_or(Quat::IDENTITY), t);
    let scl = a
        .scale
        .unwrap_or(Vec3::ONE)
        .lerp(b.scale.unwrap_or(Vec3::ONE), t);
    (pos, rot, scl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_eval() {
        let skeleton = Skeleton {
            bones: vec![
                Bone {
                    name: "root".into(),
                    parent: None,
                    local_position: [0.0; 3],
                    local_rotation: [0.0, 0.0, 0.0, 1.0],
                    local_scale: [1.0; 3],
                    inverse_bind: Mat4::IDENTITY.to_cols_array_2d(),
                },
                Bone {
                    name: "arm".into(),
                    parent: Some(0),
                    local_position: [1.0, 0.0, 0.0],
                    local_rotation: [0.0, 0.0, 0.0, 1.0],
                    local_scale: [1.0; 3],
                    inverse_bind: Mat4::IDENTITY.to_cols_array_2d(),
                },
            ],
        };
        let clip = AnimationClip {
            name: "idle".into(),
            duration: 1.0,
            tracks: vec![],
            looping: true,
        };
        let joints = skeleton.joint_matrices(&clip, 0.0);
        assert_eq!(joints.len(), 2);
    }

    #[test]
    fn test_joint_constraint_clamp() {
        let d = std::f32::consts::PI / 180.0;
        let constraint = JointConstraint {
            min: [-30.0 * d, -30.0 * d, -30.0 * d],
            max: [30.0 * d, 30.0 * d, 30.0 * d],
        };
        // Rotation within limits should pass through unchanged (approximately)
        let small = Quat::from_rotation_x(10.0 * d);
        let clamped = constraint.clamp(small);
        let (cx, _, _) = quat_to_euler_xyz(clamped);
        assert!((cx - 10.0 * d).abs() < 0.01);

        // Rotation exceeding limits should be clamped
        let big = Quat::from_rotation_x(90.0 * d);
        let clamped = constraint.clamp(big);
        let (cx, _, _) = quat_to_euler_xyz(clamped);
        assert!((cx - 30.0 * d).abs() < 0.01);
    }

    #[test]
    fn test_default_humanoid_constraints() {
        let constraints = default_humanoid_constraints();
        assert!(constraints.len() >= 13);
        // Verify head constraint exists with expected range
        let (name, c) = &constraints[0];
        assert_eq!(*name, "head");
        let d = std::f32::consts::PI / 180.0;
        assert!((c.max[0] - 60.0 * d).abs() < 0.001);
    }

    #[test]
    fn test_evaluate_constrained() {
        let d = std::f32::consts::PI / 180.0;
        let skeleton = Skeleton {
            bones: vec![
                Bone {
                    name: "root".into(),
                    parent: None,
                    local_position: [0.0; 3],
                    local_rotation: [0.0, 0.0, 0.0, 1.0],
                    local_scale: [1.0; 3],
                    inverse_bind: Mat4::IDENTITY.to_cols_array_2d(),
                },
                Bone {
                    name: "head".into(),
                    parent: Some(0),
                    local_position: [0.0, 1.0, 0.0],
                    local_rotation: [0.0, 0.0, 0.0, 1.0],
                    local_scale: [1.0; 3],
                    inverse_bind: Mat4::IDENTITY.to_cols_array_2d(),
                },
            ],
        };
        // Animate head with extreme rotation (90° X)
        let clip = AnimationClip {
            name: "extreme".into(),
            duration: 1.0,
            tracks: vec![BoneTrack {
                bone_index: 1,
                keyframes: vec![Keyframe {
                    time: 0.0,
                    position: Some(Vec3::new(0.0, 1.0, 0.0)),
                    rotation: Some(Quat::from_rotation_x(90.0 * d)),
                    scale: Some(Vec3::ONE),
                }],
            }],
            looping: false,
        };
        let constraints = skeleton.build_humanoid_constraints();
        let world = skeleton.evaluate_constrained(&clip, 0.0, &constraints);
        assert_eq!(world.len(), 2);
        // Head should be clamped — verify it differs from unconstrained
        let unconstrained = skeleton.evaluate(&clip, 0.0);
        assert_ne!(
            world[1].to_cols_array_2d(),
            unconstrained[1].to_cols_array_2d()
        );
    }
}
