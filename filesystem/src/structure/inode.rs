#[repr(u32)]
#[derive(Copy, Clone)]
enum InodeTag {
    Standard = 0,
    EntitiesList = 10,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Inode {
    val: u32,
    tag: InodeTag,
}

// TODO free list
pub struct InodePool {
    next: Inode,
}

impl Inode {
    pub fn standard(val: u32) -> Self {
        Self {
            tag: InodeTag::Standard,
            val,
        }
    }
}

impl InodePool {
    pub fn new_dynamic(next: u64) -> Self {
        Self {
            next: Inode {
                tag: InodeTag::EntitiesList,
                val: next as u32,
            },
        }
    }

    pub fn allocate(&mut self) -> Inode {
        let new = self.next;
        self.next.val += 1;
        new
    }
}

impl From<u64> for Inode {
    fn from(u: u64) -> Self {
        unsafe { std::mem::transmute(u) }
    }
}

impl From<Inode> for u64 {
    fn from(i: Inode) -> Self {
        unsafe { std::mem::transmute(i) }
    }
}
