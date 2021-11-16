use std::collections::VecDeque;

const INODE_BLOCK_SIZE: u64 = 2048;

pub struct InodeBlock {
    next: u64,
    end: u64,
}

pub struct InodeBlockAllocator {
    free_blocks: VecDeque<InodeBlock>,
    next_block_start: u64,
}

impl Default for InodeBlockAllocator {
    fn default() -> Self {
        Self {
            free_blocks: VecDeque::default(),
            next_block_start: 1,
        }
    }
}

impl InodeBlockAllocator {
    pub fn allocate(&mut self) -> InodeBlock {
        if let Some(top) = self.free_blocks.pop_front() {
            return top;
        }

        let new_block = InodeBlock {
            next: self.next_block_start,
            end: self.next_block_start + INODE_BLOCK_SIZE,
        };
        self.next_block_start += INODE_BLOCK_SIZE;
        new_block
    }

    pub fn free(&mut self, block: InodeBlock) {
        self.free_blocks.push_back(block)
    }
}

impl Iterator for InodeBlock {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.end {
            None
        } else {
            let inode = self.next;
            self.next += 1;
            Some(inode)
        }
    }
}
