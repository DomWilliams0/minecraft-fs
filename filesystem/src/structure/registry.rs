use crate::state::{GameState, GameStateInterest};
use crate::structure::inode::InodePool;
use crate::structure::structure::Root;
use ipc::ReadCommand;
use parking_lot::{Mutex, MutexGuard};
use smallvec::SmallVec;
use std::any::{Any, TypeId};
use std::borrow::Cow;

use std::collections::HashMap;
use std::ffi::OsStr;

use std::ops::Deref;
use std::os::unix::ffi::OsStrExt;
use std::sync::Arc;

pub struct FilesystemStructure {
    // TODO faster lookup structure and/or perfect hash
    /// Inode to entry
    registry: HashMap<u64, FilesystemEntry>,

    // TODO perfect hash
    /// Type to inode
    ty_registry: HashMap<TypeId, u64>,

    /// (parent inode, first 4 bytes of osstr file name) -> children inodes
    child_registry: HashMap<(u64, u32), SmallVec<[u64; 2]>>,

    /// parent inode -> dynamic entries inodes
    dynamic_child_registry: Arc<Mutex<HashMap<u64, Vec<u64>>>>,
}

pub struct FilesystemEntry {
    name: Cow<'static, OsStr>,
    entry: Entry,
}

pub enum Entry {
    File(Box<dyn FileEntry>),
    Dir(Box<dyn DirEntry>),
}

#[derive(Clone)]
pub enum EntryRef {
    File(&'static dyn FileEntry),
    Dir(&'static dyn DirEntry),
}

#[allow(unused_variables)]
pub trait DirEntry: Send + Sync + Any {
    fn children(&self) -> &'static [EntryRef];

    fn dynamic_children(&self, children_out: &mut Vec<FilesystemEntry>, state: &GameState) {}

    fn filter(&self, state: &GameState) -> EntryFilterResult {
        EntryFilterResult::IncludeSelf
    }

    fn register_interest(&self, interest: &mut GameStateInterest) {}
}

#[allow(unused_variables)]
pub trait FileEntry: Send + Sync + Any {
    fn read(&self) -> Option<ReadCommand>;

    fn should_include(&self, state: &GameState) -> bool {
        true
    }
}

struct StructureBuilder {
    registry: HashMap<u64, FilesystemEntry>,
    ty_registry: HashMap<TypeId, u64>,
}

pub struct Registration {
    pub name: &'static str,
    pub entry_fn: fn() -> Entry,
}

inventory::collect!(Registration);

#[derive(Copy, Clone)]
pub enum EntryFilterResult {
    IncludeSelf,
    IncludeAllChildren,
    Exclude,
}

pub enum DynamicChildrenCreation {
    None,
    AlreadyExisted,
    NewlyCreated(Vec<(u64, FilesystemEntry)>),
}

impl StructureBuilder {
    fn register(&mut self, inode: u64, name: &'static str, instance: Entry) {
        log::debug!("registering inode={}, name={:?}", inode, name);
        let ty = instance.entry_typeid();

        assert!(
            self.registry
                .insert(
                    inode,
                    FilesystemEntry {
                        name: Cow::Borrowed(name.as_ref()),
                        entry: instance,
                    },
                )
                .is_none(),
            "duplicate entry for inode {} ({:?})",
            inode,
            name
        );

        assert!(
            self.ty_registry.insert(ty, inode).is_none(),
            "duplicate type registration for {:?}",
            name
        );
    }

    fn finish(self) -> FilesystemStructure {
        let mut child_registry = HashMap::new();
        populate_children(
            &self.registry,
            &self.ty_registry,
            &mut child_registry,
            self.registry.keys().copied(),
        );

        let dynamic_registry = Arc::new(Mutex::new(HashMap::new()));

        FilesystemStructure {
            registry: self.registry,
            ty_registry: self.ty_registry,
            child_registry,
            dynamic_child_registry: dynamic_registry,
        }
    }
}
fn populate_children(
    registry: &HashMap<u64, FilesystemEntry>,
    ty_registry: &HashMap<TypeId, u64>,
    child_registry: &mut HashMap<(u64, u32), SmallVec<[u64; 2]>>,
    to_register: impl Iterator<Item = u64>,
) {
    for parent_inode in to_register {
        let entry = registry
            .get(&parent_inode)
            .expect("unregistered parent inode");
        if let Entry::Dir(dir) = &entry.entry {
            for child in dir.children() {
                let (child_inode, child) = lookup_entry(child, registry, ty_registry);
                let child_prefix = name_prefix(child.name());

                use std::collections::hash_map::Entry as MapEntry;
                let children = match child_registry.entry((parent_inode, child_prefix)) {
                    MapEntry::Occupied(entries) => entries.into_mut(),
                    MapEntry::Vacant(entry) => entry.insert(SmallVec::new()),
                };

                children.push(child_inode)
            }
        }
    }
}
fn name_prefix(s: &OsStr) -> u32 {
    let mut buf: [u8; 4] = [0; 4];

    let src = s.as_bytes();
    let prefix_len = src.len().min(4);
    let dst = &mut buf[..prefix_len];
    dst.copy_from_slice(&src[..prefix_len]);
    u32::from_ne_bytes(buf)
}

fn skip_prefix(s: &OsStr) -> &[u8] {
    let s = s.as_bytes();

    if s.len() <= 4 {
        s
    } else {
        &s[4..]
    }
}

impl FilesystemStructure {
    pub fn new() -> (Self, InodePool) {
        let mut builder = StructureBuilder {
            registry: HashMap::new(),
            ty_registry: HashMap::new(),
        };

        let mut next_inode = 2;
        for reg in inventory::iter::<Registration> {
            let my_inode = if reg.name.is_empty() {
                // must be root
                1
            } else {
                let curr = next_inode;
                next_inode += 1;
                curr
            };

            builder.register(my_inode, reg.name, (reg.entry_fn)());
        }

        let root = builder.registry.get(&1).expect("missing root dir");
        assert_eq!(
            root.entry.entry_typeid(),
            TypeId::of::<Root>(),
            "wrong root"
        );

        (builder.finish(), InodePool::new_dynamic(next_inode))
    }

    pub fn lookup_inode(&self, inode: u64) -> Option<&Entry> {
        self.registry.get(&inode).map(|e| &e.entry)
    }

    pub fn lookup_inode_entry<'a>(
        &'a self,
        inode: u64,
        dynamic_children: &'a DynamicChildrenCreation,
    ) -> Option<&'a FilesystemEntry> {
        self.registry.get(&inode).or_else(|| {
            // if dynamic children were just created, they are not yet in the registry, so
            // search manually
            if let DynamicChildrenCreation::NewlyCreated(new) = dynamic_children {
                new.iter()
                    .find_map(|(i, e)| if *i == inode { Some(e) } else { None })
            } else {
                None
            }
        })
    }

    pub fn lookup_entry(&self, entry: &EntryRef) -> &FilesystemEntry {
        lookup_entry(entry, &self.registry, &self.ty_registry).1
    }

    pub fn lookup_child(&self, parent: u64, name: &OsStr) -> Option<(u64, &Entry)> {
        let child_prefix = name_prefix(name);

        // check static children first
        self.child_registry
            .get(&(parent, child_prefix))
            .and_then(|children| {
                children
                    .iter()
                    .map(|inode| {
                        let child = self.registry.get(inode).expect("unregistered inode");
                        (*inode, child)
                    })
                    .find_map(|(inode, child)| {
                        // already compared first 4 bytes
                        let a = skip_prefix(name);
                        let b = skip_prefix(child.name());
                        if a == b {
                            Some((inode, &child.entry))
                        } else {
                            None
                        }
                    })
            })
            .or_else(|| {
                // check dynamic children
                let reg = self.dynamic_child_registry.lock();
                reg.get(&parent).and_then(|children| {
                    children
                        .iter()
                        .map(|inode| {
                            (
                                *inode,
                                self.registry.get(inode).expect("unregistered child inode"),
                            )
                        })
                        .find_map(|(inode, e)| {
                            if e.name == name {
                                Some((inode, &e.entry))
                            } else {
                                None
                            }
                        })
                })
            })
    }

    pub fn dynamic_children(
        &self,
        parent: u64,
        inodes: &mut InodePool,
        state: &GameState,
    ) -> (impl Deref<Target = [u64]> + '_, DynamicChildrenCreation) {
        use std::collections::hash_map::Entry;

        let reg = self.dynamic_child_registry.lock();
        let mut created = DynamicChildrenCreation::None;
        let inodes = MutexGuard::map(reg, |reg| {
            match reg.entry(parent) {
                Entry::Occupied(e) => {
                    created = DynamicChildrenCreation::AlreadyExisted;
                    &mut e.into_mut()[..]
                }
                Entry::Vacant(e) => {
                    let mut vec = Vec::new();
                    let parent_dir = self
                        .registry
                        .get(&parent)
                        .and_then(|e| e.entry.as_dir())
                        .unwrap(); // must be valid to get this far
                    parent_dir.dynamic_children(&mut vec, state);

                    if vec.is_empty() {
                        // no dynamics
                        created = DynamicChildrenCreation::None;
                        &mut []
                    } else {
                        let child_inodes = vec
                            .iter()
                            .map(|_| u64::from(inodes.allocate()))
                            .collect::<Vec<_>>();
                        let child_entries =
                            child_inodes.iter().copied().zip(vec).collect::<Vec<_>>();
                        created = DynamicChildrenCreation::NewlyCreated(child_entries);
                        &mut e.insert(child_inodes)[..]
                    }
                }
            }
        });

        (inodes, created)
    }

    pub fn register_dynamic_children(&mut self, parent: u64, children: DynamicChildrenCreation) {
        if let DynamicChildrenCreation::NewlyCreated(entries) = children {
            let child_inodes = entries.iter().map(|(i, _)| i).copied().collect::<Vec<_>>();
            log::trace!(
                "registering dynamic children for parent {}: {:?}",
                parent,
                child_inodes
            );
            for (inode, e) in entries {
                self.registry.insert(inode, e);
            }

            // register static grandchildren for the dynamic children
            populate_children(
                &self.registry,
                &self.ty_registry,
                &mut self.child_registry,
                child_inodes.into_iter(),
            );
        }
    }
}
fn lookup_entry<'a>(
    entry: &EntryRef,
    registry: &'a HashMap<u64, FilesystemEntry>,
    ty_registry: &'a HashMap<TypeId, u64>,
) -> (u64, &'a FilesystemEntry) {
    let inode = ty_registry
        .get(&entry.entry_typeid())
        .unwrap_or_else(|| panic!("unregistered type for inode {:?}", entry.entry_typeid()));

    let entry = registry
        .get(inode)
        .unwrap_or_else(|| panic!("unregistered inode {}", inode));

    (*inode, entry)
}

impl Entry {
    pub fn file<T: FileEntry + 'static>(t: T) -> Self {
        Entry::File(Box::new(t))
    }

    pub fn dir<T: DirEntry + 'static>(t: T) -> Self {
        Entry::Dir(Box::new(t))
    }

    fn entry_typeid(&self) -> TypeId {
        match self {
            Entry::File(b) => box_typeid(b),
            Entry::Dir(b) => box_typeid(b),
        }
    }

    pub fn as_dir(&self) -> Option<&dyn DirEntry> {
        match self {
            Entry::Dir(dir) => Some(&**dir),
            _ => None,
        }
    }
}

impl EntryRef {
    fn entry_typeid(&self) -> TypeId {
        match self {
            EntryRef::File(b) => (*b).type_id(),
            EntryRef::Dir(b) => (*b).type_id(),
        }
    }

    pub fn filter(&self, state: &GameState) -> EntryFilterResult {
        match self {
            EntryRef::File(b) => {
                if b.should_include(state) {
                    EntryFilterResult::IncludeSelf
                } else {
                    EntryFilterResult::Exclude
                }
            }
            EntryRef::Dir(b) => b.filter(state),
        }
    }
}

impl FilesystemEntry {
    pub fn name(&self) -> &OsStr {
        &self.name
    }

    pub fn kind(&self) -> fuser::FileType {
        match self.entry {
            Entry::File(_) => fuser::FileType::RegularFile,
            Entry::Dir(_) => fuser::FileType::Directory,
        }
    }

    pub fn new(name: impl Into<Cow<'static, OsStr>>, entry: Entry) -> Self {
        Self {
            name: name.into(),
            entry,
        }
    }
}

#[allow(clippy::borrowed_box)]
fn box_typeid<T: ?Sized + Any>(b: &Box<T>) -> TypeId {
    Any::type_id(&**b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typeid() {
        struct A;
        struct B;

        let a = Box::new(A) as Box<dyn Any>;
        let a2 = Box::new(A) as Box<dyn Any>;
        let b = Box::new(B) as Box<dyn Any>;

        assert_eq!(box_typeid(&a), box_typeid(&a2));
        assert_ne!(box_typeid(&a), box_typeid(&b));
        assert_ne!(box_typeid(&a2), box_typeid(&b));
    }
}
