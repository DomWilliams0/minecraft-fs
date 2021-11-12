use crate::structure::structure::Root;
use ipc::CommandType;
use smallvec::SmallVec;
use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

pub struct FilesystemStructure {
    // TODO faster lookup structure and/or perfect hash
    /// Inode to entry
    registry: HashMap<u64, FilesystemEntry>,

    // TODO perfect hash
    /// Type to inode
    ty_registry: HashMap<TypeId, u64>,

    /// (parent inode, first 4 bytes of osstr file name) -> children inodes
    child_registry: HashMap<(u64, u32), SmallVec<[u64; 2]>>,
}

pub struct FilesystemEntry {
    name: Cow<'static, OsStr>,
    entry: Entry,
}

pub enum Entry {
    File(Box<dyn FileEntry>),
    Dir(Box<dyn DirEntry>),
}

pub enum EntryRef {
    File(&'static dyn FileEntry),
    Dir(&'static dyn DirEntry),
}

pub trait DirEntry: Send + Sync + Any {
    fn children(&self) -> &'static [EntryRef];
}

pub trait FileEntry: Send + Sync + Any {
    fn read_command(&self) -> Option<CommandType>;
}

struct StructureBuilder {
    registry: HashMap<u64, FilesystemEntry>,
    ty_registry: HashMap<TypeId, u64>,
}

pub struct Registration {
    pub name: &'static str,
    pub children: &'static [EntryRef],
    pub entry_fn: fn() -> Entry,
}

inventory::collect!(Registration);

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

        for (parent_inode, entry) in self.registry.iter() {
            if let Entry::Dir(dir) = &entry.entry {
                for child in dir.children() {
                    let (child_inode, child) =
                        lookup_entry(child, &self.registry, &self.ty_registry);
                    let child_prefix = name_prefix(child.name());

                    use std::collections::hash_map::Entry as MapEntry;
                    let children = match child_registry.entry((*parent_inode, child_prefix)) {
                        MapEntry::Occupied(entries) => entries.into_mut(),
                        MapEntry::Vacant(entry) => entry.insert(SmallVec::new()),
                    };

                    children.push(child_inode)
                }
            }
        }

        FilesystemStructure {
            registry: self.registry,
            ty_registry: self.ty_registry,
            child_registry,
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
    pub fn new() -> Self {
        let mut builder = StructureBuilder {
            registry: HashMap::new(),
            ty_registry: HashMap::new(),
        };

        let mut inode = 2;
        for reg in inventory::iter::<Registration> {
            let my_inode = if reg.name.is_empty() {
                // must be root
                1
            } else {
                let curr = inode;
                inode += 1;
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

        builder.finish()
    }

    pub fn lookup_inode(&self, inode: u64) -> Option<&Entry> {
        self.registry.get(&inode).map(|e| &e.entry)
    }

    pub fn lookup_entry(&self, entry: &EntryRef) -> &FilesystemEntry {
        lookup_entry(entry, &self.registry, &self.ty_registry).1
    }

    pub fn lookup_child(&self, parent: u64, name: &OsStr) -> Option<(u64, &Entry)> {
        let child_prefix = name_prefix(name);

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
}

impl EntryRef {
    fn entry_typeid(&self) -> TypeId {
        match self {
            EntryRef::File(b) => (*b).type_id(),
            EntryRef::Dir(b) => (*b).type_id(),
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
