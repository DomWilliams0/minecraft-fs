const STATIC_MAX: u64 = 5000;

pub struct InodeBlockAllocator {
    next_dyn: u64,
    next_static: u64,
}

impl Default for InodeBlockAllocator {
    fn default() -> Self {
        Self {
            next_static: 1,
            next_dyn: STATIC_MAX,
        }
    }
}

impl InodeBlockAllocator {
    pub fn allocate_static(&mut self) -> u64 {
        assert!(
            self.next_static <= STATIC_MAX,
            "exhausted static inodes, increase limit"
        );
        let new = self.next_static + 1;
        std::mem::replace(&mut self.next_static, new)
    }

    pub fn allocate(&mut self) -> u64 {
        let new = self.next_dyn + 1;
        std::mem::replace(&mut self.next_dyn, new)
    }
}

pub fn is_inode_static(inode: u64) -> bool {
    inode < STATIC_MAX
}
