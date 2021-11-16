use crate::state::{GameState, GameStateInterest};
use crate::structure::inode::{InodeBlock, InodeBlockAllocator};
use ipc::generated::CommandType;
use ipc::{BodyType, CommandState};
use log::trace;
use smallvec::{smallvec, SmallVec};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::time::{Duration, Instant};

pub struct FilesystemStructure {
    inner: StructureInner,
}

pub struct FilesystemStructureBuilder {
    inner: StructureInner,
}

struct StructureInner {
    inode_alloc: InodeBlockAllocator,
    static_inodes: InodeBlock,
    root: u64,
    registry: HashMap<u64, Entry>,
    /// parent -> list of children
    child_registry: HashMap<u64, Vec<(u64, Cow<'static, str>)>>,

    /// child -> parent
    parent_registry: HashMap<u64, u64>,

    /// owning dir inode -> _
    dynamic_state: HashMap<u64, DynamicState>,
}

#[derive(Hash, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DynamicStateType {
    EntityIds = 0,
}

// TODO 1 second
const STATE_TTL: Duration = Duration::from_secs(10000000);

struct DynamicState {
    inodes: SmallVec<[InodeBlock; 1]>,
    time_collected: Instant,
}

pub enum Entry {
    File(FileEntry),
    Dir(DirEntry),
    Link(LinkEntry),
}

pub struct DynamicDirRegistrationer {
    /// (inode, name, entry, parent)
    entries: Vec<(u64, Cow<'static, str>, Entry, Option<u64>)>,
    inodes: InodeBlock,
}
pub type DynamicDirFn = fn(&GameState, &mut DynamicDirRegistrationer);

pub type FileFilterFn = fn(&GameState) -> bool;
pub type DirFilterFn = fn(&GameState) -> EntryFilterResult;

#[derive(Default)]
pub struct DirEntry {
    dynamic: Option<(DynamicStateType, DynamicDirFn)>,
    filter: Option<DirFilterFn>,
}

#[derive(Default)]
pub struct FileEntry {
    behaviour: Option<FileBehaviour>,
    associated_data: Option<EntryAssociatedData>,
    filter: Option<FileFilterFn>,
}

pub type LinkTargetFn = fn(&GameState) -> Option<Cow<'static, str>>;
pub struct LinkEntry {
    target: LinkTargetFn,
    filter: Option<FileFilterFn>,
}

#[derive(Default)]
pub struct DirEntryBuilder(DirEntry);
#[derive(Default)]
pub struct FileEntryBuilder(FileEntry);
pub struct LinkEntryBuilder(LinkEntry);

#[derive(Copy, Clone)]
pub enum EntryFilterResult {
    IncludeSelf,
    IncludeAllChildren,
    Exclude,
}

#[derive(Copy, Clone)]
pub enum FileBehaviour {
    ReadOnly(CommandType, BodyType),
    WriteOnly(CommandType, BodyType),
    ReadWrite(CommandType, BodyType),
    // TODO rw with different types
}

pub enum EntryAssociatedData {
    EntityId(i32),
    World(u8), // TODO
}

pub struct DynamicInterest {
    /// (inode, interest)
    inodes: SmallVec<[(u64, DynamicStateType); 2]>,
    need_fetching: HashSet<DynamicStateType>,
}

impl FilesystemStructure {
    pub fn builder() -> FilesystemStructureBuilder {
        let mut inode_alloc = InodeBlockAllocator::default();
        let mut static_inodes = inode_alloc.allocate();
        let mut registry = HashMap::with_capacity(256);

        // register root
        let root = static_inodes.next().unwrap(); // just created it
        registry.insert(root, Entry::Dir(DirEntry::default()));

        FilesystemStructureBuilder {
            inner: StructureInner {
                inode_alloc,
                static_inodes,
                root,
                registry,
                child_registry: HashMap::new(),
                parent_registry: HashMap::new(),
                dynamic_state: HashMap::new(),
            },
        }
    }

    pub fn lookup_inode(&self, inode: u64) -> Option<&Entry> {
        self.inner.registry.get(&inode)
    }

    fn get_inode(&self, inode: u64) -> &Entry {
        self.inner
            .registry
            .get(&inode)
            .unwrap_or_else(|| panic!("unregistered inode {}", inode))
    }

    pub fn lookup_child(&self, parent: u64, name: &OsStr) -> Option<(u64, &Entry)> {
        let name = name.to_str()?; // utf8 only
        self.inner.child_registry.get(&parent).and_then(|children| {
            children.iter().find_map(|(ino, child_name)| {
                if child_name == name {
                    Some((*ino, self.get_inode(*ino)))
                } else {
                    None
                }
            })
        })
    }

    pub fn lookup_children(&self, inode: u64) -> Option<impl Iterator<Item = (&Entry, &str)> + '_> {
        self.inner.child_registry.get(&inode).map(|v| {
            v.iter()
                .map(|(inode, name)| (self.get_inode(*inode), name.as_ref()))
        })
    }

    /// Walks the hierarchy to find dirs with dynamic interest, and produces
    /// an interest for the whole hierarchy
    pub fn interest_for_inode(&self, inode: u64) -> DynamicInterest {
        let mut dynamics_required = SmallVec::new();

        self.walk_ancestors(inode, |ancestor| {
            let entry = self.get_inode(ancestor);
            if let Entry::Dir(dir) = entry {
                if let Some((interest, _)) = dir.dynamic {
                    dynamics_required.push((ancestor, interest));
                }
            }
        });

        let mut need_fetching = HashSet::new();
        for (inode, interest) in dynamics_required.iter() {
            if let Some(state) = self.inner.dynamic_state.get(inode) {
                if state.time_collected.elapsed() <= STATE_TTL {
                    // cache is valid
                    continue;
                }
            }

            need_fetching.insert(*interest);
        }

        DynamicInterest {
            inodes: dynamics_required,
            need_fetching,
        }
    }

    pub fn ensure_generated(&mut self, state: &GameState, dynamics: DynamicInterest) {
        for (inode, interest) in dynamics
            .inodes
            .iter()
            .filter(|(_, ty)| dynamics.need_fetching.contains(ty))
        {
            let dyn_fn = match self
                .lookup_inode(*inode)
                .and_then(|e| e.as_dir())
                .and_then(|dir| dir.dynamic)
            {
                Some((int, dyn_fn)) if int == *interest => dyn_fn,
                _ => {
                    log::warn!("inode {} is not a dynamic dir", inode);
                    continue;
                }
            };

            let mut registrationer = DynamicDirRegistrationer {
                inodes: self.inner.inode_alloc.allocate(),
                entries: Vec::new(),
            };

            (dyn_fn)(state, &mut registrationer);

            // register dynamic entries
            for (new_inode, new_name, new_entry, new_parent) in registrationer.entries {
                let new_parent = new_parent.unwrap_or(*inode);
                self.inner
                    .register(new_inode, new_entry, Some((new_parent, new_name)));
            }

            let state = DynamicState {
                inodes: smallvec![registrationer.inodes],
                time_collected: Instant::now(),
            };

            let prev = self.inner.dynamic_state.insert(*inode, state);
            if let Some(prev) = prev {
                // TODO clean up old inodes
            }
        }
    }

    fn walk_ancestors(&self, child: u64, mut per_parent: impl FnMut(u64)) {
        per_parent(child);

        let mut current = child;
        while let Some(parent) = self.inner.parent_registry.get(&current) {
            per_parent(*parent);
            current = *parent;
        }
    }
}

impl DynamicInterest {
    pub fn as_interest(&self) -> GameStateInterest {
        let mut interest = GameStateInterest::default();
        for dynamic in self.need_fetching.iter() {
            match dynamic {
                DynamicStateType::EntityIds => {
                    interest.entities_by_id = true;
                }
            }
        }

        interest
    }
}

impl FilesystemStructureBuilder {
    pub fn root(&self) -> u64 {
        self.inner.root
    }

    pub fn add_static_entry(
        &mut self,
        parent: u64,
        name: &'static str,
        entry: impl Into<Entry>,
    ) -> u64 {
        self.new_static_with_opt_parent(entry.into(), Some((parent, name)))
    }

    fn new_static_with_opt_parent(
        &mut self,
        entry: Entry,
        parent_info: Option<(u64, &'static str)>,
    ) -> u64 {
        let inode = self
            .inner
            .static_inodes
            .next()
            .expect("exhausted static inodes");

        self.inner.register(inode, entry, parent_info);

        inode
    }

    pub fn finish(self) -> FilesystemStructure {
        FilesystemStructure { inner: self.inner }
    }
}

impl StructureInner {
    fn register(
        &mut self,
        inode: u64,
        entry: Entry,
        parent_info: Option<(u64, impl Into<Cow<'static, str>>)>,
    ) {
        self.registry.insert(inode, entry);

        if let Some((parent, name)) = parent_info {
            let name = name.into();
            trace!(
                "registered inode {} under parent {} with name '{}'",
                inode,
                parent,
                name
            );
            self.add_child_to_parent(parent, name, inode)
        } else {
            trace!("registered inode {}", inode);
        }
    }

    fn add_child_to_parent(&mut self, parent: u64, child_name: Cow<'static, str>, child: u64) {
        use std::collections::hash_map::Entry;
        let children = match self.child_registry.entry(parent) {
            Entry::Occupied(entries) => entries.into_mut(),
            Entry::Vacant(entry) => entry.insert(Vec::new()),
        };

        children.push((child, child_name));

        if self.parent_registry.insert(child, parent).is_some() {
            panic!("multiple parents for child {}", child);
        }
    }
}

impl Entry {
    fn as_dir(&self) -> Option<&DirEntry> {
        match self {
            Entry::Dir(dir) => Some(dir),
            _ => None,
        }
    }
}

impl FileEntryBuilder {
    pub fn behaviour(mut self, behaviour: FileBehaviour) -> Self {
        self.0.behaviour = Some(behaviour);
        self
    }

    pub fn associated_data(mut self, data: EntryAssociatedData) -> Self {
        self.0.associated_data = Some(data);
        self
    }

    pub fn filter(mut self, filter: FileFilterFn) -> Self {
        self.0.filter = Some(filter);
        self
    }

    pub fn finish(self) -> FileEntry {
        self.0
    }
}

impl FileEntry {
    pub fn build() -> FileEntryBuilder {
        FileEntryBuilder::default()
    }

    pub fn behaviour(&self) -> Option<FileBehaviour> {
        self.behaviour
    }

    pub fn command_state(&self) -> CommandState {
        let mut state = CommandState::default();
        if let Some(associated) = self.associated_data.as_ref() {
            associated.apply_to_state(&mut state);
        }

        state
    }
}

impl LinkEntryBuilder {
    pub fn new(target: LinkTargetFn) -> Self {
        Self(LinkEntry {
            target,
            filter: None,
        })
    }

    pub fn filter(mut self, filter: FileFilterFn) -> Self {
        self.0.filter = Some(filter);
        self
    }

    pub fn finish(self) -> LinkEntry {
        self.0
    }
}

impl LinkEntry {
    pub fn build(target: LinkTargetFn) -> LinkEntryBuilder {
        LinkEntryBuilder::new(target)
    }

    pub fn target(&self) -> LinkTargetFn {
        self.target
    }
}

impl DirEntry {
    pub fn build() -> DirEntryBuilder {
        DirEntryBuilder::default()
    }
}

impl DirEntryBuilder {
    pub fn dynamic(mut self, ty: DynamicStateType, dyn_fn: DynamicDirFn) -> Self {
        self.0.dynamic = Some((ty, dyn_fn));
        self
    }

    pub fn filter(mut self, filter: DirFilterFn) -> Self {
        self.0.filter = Some(filter);
        self
    }

    pub fn finish(self) -> DirEntry {
        self.0
    }
}

impl EntryAssociatedData {
    fn apply_to_state(&self, state: &mut CommandState) {
        match self {
            EntryAssociatedData::EntityId(id) => state.target_entity = Some(*id),
            EntryAssociatedData::World(_) => todo!(),
        }
    }
}

impl From<FileEntry> for Entry {
    fn from(e: FileEntry) -> Self {
        Self::File(e)
    }
}

impl From<DirEntry> for Entry {
    fn from(e: DirEntry) -> Self {
        Self::Dir(e)
    }
}

impl From<LinkEntry> for Entry {
    fn from(e: LinkEntry) -> Self {
        Self::Link(e)
    }
}

impl Entry {
    pub fn filter(&self, state: &GameState) -> EntryFilterResult {
        match self {
            Entry::File(f) => {
                if let Some(filter) = f.filter {
                    return if (filter)(state) {
                        EntryFilterResult::IncludeSelf
                    } else {
                        EntryFilterResult::Exclude
                    };
                }
            }
            Entry::Link(l) => {
                if let Some(filter) = l.filter {
                    return if (filter)(state) {
                        EntryFilterResult::IncludeSelf
                    } else {
                        EntryFilterResult::Exclude
                    };
                }
            }
            Entry::Dir(d) => {
                if let Some(filter) = d.filter {
                    return (filter)(state);
                }
            }
        };
        EntryFilterResult::IncludeSelf
    }
}

impl DynamicDirRegistrationer {
    pub fn add_root_entry(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        entry: impl Into<Entry>,
    ) -> u64 {
        self.add_entry(None, name.into(), entry.into())
    }

    pub fn add_static_entry(
        &mut self,
        parent: u64,
        name: impl Into<Cow<'static, str>>,
        entry: impl Into<Entry>,
    ) -> u64 {
        self.add_entry(Some(parent), name.into(), entry.into())
    }

    fn add_entry(&mut self, parent: Option<u64>, name: Cow<'static, str>, entry: Entry) -> u64 {
        let inode = self.inodes.next().expect("exhausted dynamic inodes"); // TODO allocate more
        self.entries.push((inode, name, entry, parent));
        inode
    }
}
