use std::collections::VecDeque;

use log::trace;

const INODE_BLOCK_SIZE: u64 = 4096;

pub struct InodeBlock {
    start: u64,
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
        if let Some(mut top) = self.free_blocks.pop_front() {
            top.next = top.start;
            trace!("allocating new inode block {} - from freelist", top.start);
            return top;
        }

        let new_block = InodeBlock {
            start: self.next_block_start,
            next: self.next_block_start,
            end: self.next_block_start + INODE_BLOCK_SIZE,
        };
        self.next_block_start += INODE_BLOCK_SIZE;
        trace!("allocating new inode block {} - brand new", new_block.start);
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

impl InodeBlock {
    pub fn iter_allocated(&self) -> impl Iterator<Item = u64> {
        self.start..self.next
    }

    pub fn iter_all_without_allocating(&self) -> impl Iterator<Item = u64> {
        self.start..self.end
    }
}
