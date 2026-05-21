//! kami-voxel: Volume trait + Dense voxel grid.

/// Single voxel data.
#[derive(Debug, Clone, Copy, Default)]
pub struct Voxel {
    pub material: u8,
    pub color: [f32; 4],
}

impl Voxel {
    pub fn is_solid(&self) -> bool {
        self.material > 0
    }
    pub fn air() -> Self {
        Self {
            material: 0,
            color: [0.0; 4],
        }
    }
}

/// Volume access trait — all storage backends implement this.
pub trait Volume {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn depth(&self) -> u32;
    fn get(&self, x: u32, y: u32, z: u32) -> Voxel;
    fn set(&mut self, x: u32, y: u32, z: u32, voxel: Voxel);
    fn count_filled(&self) -> u32;
}

/// Dense voxel volume (W × H × D flat array).
pub struct DenseVolume {
    pub w: u32,
    pub h: u32,
    pub d: u32,
    data: Vec<Voxel>,
}

impl DenseVolume {
    pub fn new(w: u32, h: u32, d: u32) -> Self {
        Self {
            w,
            h,
            d,
            data: vec![Voxel::air(); (w * h * d) as usize],
        }
    }
}

impl Volume for DenseVolume {
    fn width(&self) -> u32 {
        self.w
    }
    fn height(&self) -> u32 {
        self.h
    }
    fn depth(&self) -> u32 {
        self.d
    }
    fn get(&self, x: u32, y: u32, z: u32) -> Voxel {
        if x >= self.w || y >= self.h || z >= self.d {
            return Voxel::air();
        }
        self.data[(z * self.h * self.w + y * self.w + x) as usize]
    }
    fn set(&mut self, x: u32, y: u32, z: u32, voxel: Voxel) {
        if x >= self.w || y >= self.h || z >= self.d {
            return;
        }
        self.data[(z * self.h * self.w + y * self.w + x) as usize] = voxel;
    }
    fn count_filled(&self) -> u32 {
        self.data.iter().filter(|v| v.is_solid()).count() as u32
    }
}

/// Sparse voxel volume (HashMap — only stores non-air voxels).
pub struct SparseVolume {
    pub w: u32,
    pub h: u32,
    pub d: u32,
    map: std::collections::HashMap<u64, Voxel>,
}

impl SparseVolume {
    pub fn new(w: u32, h: u32, d: u32) -> Self {
        Self {
            w,
            h,
            d,
            map: std::collections::HashMap::new(),
        }
    }
    fn key(x: u32, y: u32, z: u32) -> u64 {
        (z as u64) << 32 | (y as u64) << 16 | x as u64
    }
    /// Iterate only over filled voxels (sparse-specific optimization).
    pub fn filled_iter(&self) -> impl Iterator<Item = (u32, u32, u32, &Voxel)> {
        self.map.iter().map(|(&k, v)| {
            let x = (k & 0xFFFF) as u32;
            let y = ((k >> 16) & 0xFFFF) as u32;
            let z = ((k >> 32) & 0xFFFF) as u32;
            (x, y, z, v)
        })
    }
}

impl Volume for SparseVolume {
    fn width(&self) -> u32 {
        self.w
    }
    fn height(&self) -> u32 {
        self.h
    }
    fn depth(&self) -> u32 {
        self.d
    }
    fn get(&self, x: u32, y: u32, z: u32) -> Voxel {
        self.map
            .get(&Self::key(x, y, z))
            .copied()
            .unwrap_or(Voxel::air())
    }
    fn set(&mut self, x: u32, y: u32, z: u32, v: Voxel) {
        let k = Self::key(x, y, z);
        if v.is_solid() {
            self.map.insert(k, v);
        } else {
            self.map.remove(&k);
        }
    }
    fn count_filled(&self) -> u32 {
        self.map.len() as u32
    }
}

/// Octree voxel volume (adaptive resolution, power-of-2 size).
pub struct OctreeVolume {
    pub size: u32,
    root: OctreeNode,
}

enum OctreeNode {
    Empty,
    Leaf(Voxel),
    Branch(Box<[OctreeNode; 8]>),
}

impl OctreeVolume {
    pub fn new(size: u32) -> Self {
        assert!(size.is_power_of_two(), "octree size must be power of 2");
        Self {
            size,
            root: OctreeNode::Empty,
        }
    }
    fn insert_node(node: &mut OctreeNode, x: u32, y: u32, z: u32, size: u32, v: Voxel) {
        if size == 1 {
            *node = OctreeNode::Leaf(v);
            return;
        }
        let half = size / 2;
        let idx =
            ((x >= half) as usize) | (((y >= half) as usize) << 1) | (((z >= half) as usize) << 2);
        match node {
            OctreeNode::Empty | OctreeNode::Leaf(_) => {
                let mut children: [OctreeNode; 8] = std::array::from_fn(|_| OctreeNode::Empty);
                if let OctreeNode::Leaf(existing) = node {
                    for c in &mut children {
                        *c = OctreeNode::Leaf(*existing);
                    }
                }
                Self::insert_node(&mut children[idx], x % half, y % half, z % half, half, v);
                *node = OctreeNode::Branch(Box::new(children));
            }
            OctreeNode::Branch(children) => {
                Self::insert_node(&mut children[idx], x % half, y % half, z % half, half, v);
            }
        }
    }
    fn query_node(node: &OctreeNode, x: u32, y: u32, z: u32, size: u32) -> Voxel {
        match node {
            OctreeNode::Empty => Voxel::air(),
            OctreeNode::Leaf(v) => *v,
            OctreeNode::Branch(children) => {
                let half = size / 2;
                if half == 0 {
                    return Voxel::air();
                }
                let idx = ((x >= half) as usize)
                    | (((y >= half) as usize) << 1)
                    | (((z >= half) as usize) << 2);
                Self::query_node(&children[idx], x % half, y % half, z % half, half)
            }
        }
    }
}

impl Volume for OctreeVolume {
    fn width(&self) -> u32 {
        self.size
    }
    fn height(&self) -> u32 {
        self.size
    }
    fn depth(&self) -> u32 {
        self.size
    }
    fn get(&self, x: u32, y: u32, z: u32) -> Voxel {
        Self::query_node(&self.root, x, y, z, self.size)
    }
    fn set(&mut self, x: u32, y: u32, z: u32, v: Voxel) {
        Self::insert_node(&mut self.root, x, y, z, self.size, v);
    }
    fn count_filled(&self) -> u32 {
        let mut c = 0u32;
        for z in 0..self.size {
            for y in 0..self.size {
                for x in 0..self.size {
                    if self.get(x, y, z).is_solid() {
                        c += 1;
                    }
                }
            }
        }
        c
    }
}

/// Legacy compat wrapper.
pub struct VoxelVolume {
    pub w: u32,
    pub h: u32,
    pub d: u32,
    inner: VoxelInner,
}
enum VoxelInner {
    Dense(DenseVolume),
    Sparse(SparseVolume),
    Octree(OctreeVolume),
}

impl VoxelVolume {
    pub fn new_dense(w: u32, h: u32, d: u32) -> Self {
        Self {
            w,
            h,
            d,
            inner: VoxelInner::Dense(DenseVolume::new(w, h, d)),
        }
    }
    pub fn new_sparse(w: u32, h: u32, d: u32) -> Self {
        Self {
            w,
            h,
            d,
            inner: VoxelInner::Sparse(SparseVolume::new(w, h, d)),
        }
    }
    pub fn new_octree(size: u32) -> Self {
        Self {
            w: size,
            h: size,
            d: size,
            inner: VoxelInner::Octree(OctreeVolume::new(size)),
        }
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> Voxel {
        match &self.inner {
            VoxelInner::Dense(v) => v.get(x, y, z),
            VoxelInner::Sparse(v) => v.get(x, y, z),
            VoxelInner::Octree(v) => v.get(x, y, z),
        }
    }
    pub fn set(&mut self, x: u32, y: u32, z: u32, voxel: Voxel) {
        match &mut self.inner {
            VoxelInner::Dense(v) => v.set(x, y, z, voxel),
            VoxelInner::Sparse(v) => v.set(x, y, z, voxel),
            VoxelInner::Octree(v) => v.set(x, y, z, voxel),
        }
    }
    pub fn count_filled(&self) -> u32 {
        match &self.inner {
            VoxelInner::Dense(v) => v.count_filled(),
            VoxelInner::Sparse(v) => v.count_filled(),
            VoxelInner::Octree(v) => v.count_filled(),
        }
    }
    pub fn width(&self) -> u32 {
        self.w
    }
    pub fn height(&self) -> u32 {
        self.h
    }
    pub fn depth(&self) -> u32 {
        self.d
    }

    pub fn to_sparse(&self) -> Self {
        let mut s = SparseVolume::new(self.w, self.h, self.d);
        for z in 0..self.d {
            for y in 0..self.h {
                for x in 0..self.w {
                    let v = self.get(x, y, z);
                    if v.is_solid() {
                        s.set(x, y, z, v);
                    }
                }
            }
        }
        Self {
            w: self.w,
            h: self.h,
            d: self.d,
            inner: VoxelInner::Sparse(s),
        }
    }

    pub fn storage_type(&self) -> &str {
        match &self.inner {
            VoxelInner::Dense(_) => "dense",
            VoxelInner::Sparse(_) => "sparse",
            VoxelInner::Octree(_) => "octree",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense() {
        let mut v = DenseVolume::new(4, 4, 4);
        v.set(
            1,
            2,
            3,
            Voxel {
                material: 1,
                color: [1.0; 4],
            },
        );
        assert!(v.get(1, 2, 3).is_solid());
        assert_eq!(v.count_filled(), 1);
    }

    #[test]
    fn sparse() {
        let mut v = SparseVolume::new(8, 8, 8);
        v.set(
            3,
            3,
            3,
            Voxel {
                material: 2,
                color: [0.0, 1.0, 0.0, 1.0],
            },
        );
        assert!(v.get(3, 3, 3).is_solid());
        assert_eq!(v.count_filled(), 1);
        // Sparse-specific: filled_iter
        assert_eq!(v.filled_iter().count(), 1);
    }

    #[test]
    fn octree() {
        let mut v = OctreeVolume::new(8);
        v.set(
            1,
            2,
            3,
            Voxel {
                material: 1,
                color: [1.0; 4],
            },
        );
        v.set(
            7,
            7,
            7,
            Voxel {
                material: 1,
                color: [0.0, 0.0, 1.0, 1.0],
            },
        );
        assert!(v.get(1, 2, 3).is_solid());
        assert!(v.get(7, 7, 7).is_solid());
        assert!(!v.get(0, 0, 0).is_solid());
    }

    #[test]
    fn wrapper_compat() {
        let mut vol = VoxelVolume::new_dense(4, 4, 4);
        vol.set(
            0,
            0,
            0,
            Voxel {
                material: 1,
                color: [1.0; 4],
            },
        );
        let sparse = vol.to_sparse();
        assert_eq!(sparse.count_filled(), 1);
        assert_eq!(sparse.storage_type(), "sparse");
    }
}
