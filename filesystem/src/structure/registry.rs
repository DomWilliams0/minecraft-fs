use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fmt::Debug;
use std::time::{Duration, Instant};

use log::{debug, trace, warn};
use smallvec::{smallvec, SmallVec};
use strum::{EnumIter, IntoEnumIterator};

use ipc::generated::{BlockPos, CommandType, Dimension};
use ipc::{BodyType, CommandState, TargetEntity};

use crate::state::{GameState, GameStateInterest};
use crate::structure::inode::{InodeBlock, InodeBlockAllocator};

pub struct FilesystemStructure {
    inner: StructureInner,

    last_maintain: Instant,
    // TODO use an int specific map for performance
    inode_remapping: HashMap<u64, u64>,
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
    dynamic_state: HashMap<(u64, DynamicStateType), DynamicState>,

    phantom_registry: HashMap<u64, (PhantomChildFn, PhantomDynamicInterestFn, DynamicDirFn)>,
}

pub type PhantomChildFn = fn(&str) -> Option<PhantomChildType>;

#[derive(Debug, Hash, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum DynamicStateType {
    EntityIds,
    PlayerId,
    Block([i32; 3]),
}

#[derive(Debug, Copy, Clone)]
pub enum PhantomChildType {
    Block([i32; 3]),
}

const STATE_TTL: Duration = Duration::from_secs(1);

struct DynamicState {
    inodes: SmallVec<[InodeBlock; 2]>,
    time_collected: Instant,
}

pub enum Entry {
    File(FileEntry),
    Dir(DirEntry),
    Link(LinkEntry),
}

pub struct DynamicDirRegistrationer<'a> {
    /// (inode, name, entry, parent)
    entries: Vec<(u64, Cow<'static, str>, Entry, Option<u64>)>,
    current_block: InodeBlock,
    all_blocks: SmallVec<[InodeBlock; 2]>,
    inodes: &'a mut InodeBlockAllocator,
    parent: u64,
}

pub type DynamicDirFn = fn(&GameState, &mut DynamicDirRegistrationer);

pub type PhantomDynamicInterestFn = fn(PhantomChildType) -> DynamicStateType;

pub type FileFilterFn = fn(&GameState) -> bool;
pub type DirFilterFn = fn(&GameState) -> EntryFilterResult;

#[derive(Default)]
pub struct DirEntry {
    dynamic: Option<(DynamicStateType, DynamicDirFn)>,
    associated_data: Option<EntryAssociatedData>,
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
    Static(&'static str),
    /// Not readable or writable
    ForShow,
}

pub enum EntryAssociatedData {
    PlayerId,
    EntityId(i32),
    World(Dimension),
    Block(BlockPos),
}

pub struct DynamicInterest {
    original_inode: u64,
    /// (inode, interest)
    inodes: SmallVec<[(DynamicInode, DynamicStateType); 2]>,
    need_fetching: HashSet<DynamicStateType>,
    interest: GameStateInterest,
    /// To be generated under the parent inode
    phantom: Option<DynamicPhantom>,
}

struct DynamicPhantom {
    parent: u64,
    child_name: String,
    interest: DynamicStateType,
    dir_associated_data: EntryAssociatedData,
    dyn_fn: DynamicDirFn,
}

enum DynamicInode {
    Phantom(u64),
    Inode(u64),
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
                phantom_registry: HashMap::new(),
            },
        }
    }

    pub fn lookup_inode(&self, inode: u64) -> Option<&Entry> {
        self.inner.registry.get(&self.remap_inode(inode))
    }

    fn remap_inode(&self, inode: u64) -> u64 {
        self.inode_remapping.get(&inode).copied().unwrap_or(inode)
    }

    fn try_get_inode(&self, inode: u64) -> Option<&Entry> {
        self.inner.registry.get(&self.remap_inode(inode))
    }

    // fn get_inode(&self, inode: u64) -> &Entry {
    //     self.try_get_inode(inode).unwrap_or_else(|| {
    //         panic!(
    //             "unregistered inode {} (mapped to {:?})",
    //             inode,
    //             self.inode_remapping.get(&inode)
    //         )
    //     })
    // }

    pub fn lookup_child(&self, parent: u64, name: &OsStr) -> Option<(u64, &Entry)> {
        let name = name.to_str()?; // utf8 only
        self.inner
            .child_registry
            .get(&self.remap_inode(parent))
            .and_then(|children| {
                children.iter().find_map(|(ino, child_name)| {
                    if child_name == name {
                        self.try_get_inode(*ino).map(|entry| (*ino, entry))
                    } else {
                        None
                    }
                })
            })
    }

    pub fn lookup_children(&self, inode: u64) -> Option<impl Iterator<Item = (&Entry, &str)> + '_> {
        self.inner
            .child_registry
            .get(&self.remap_inode(inode))
            .map(|v| {
                v.iter().filter_map(|(inode, name)| {
                    self.try_get_inode(*inode).map(|ino| (ino, name.as_ref()))
                })
            })
    }

    /// Walks the hierarchy to find dirs with dynamic interest, and produces
    /// an interest for the whole hierarchy
    pub fn interest_for_inode(
        &self,
        inode: u64,
        looked_up_child: Option<&OsStr>,
    ) -> DynamicInterest {
        let mut dynamics_required = SmallVec::new();
        let mut interest = GameStateInterest::default();

        self.walk_ancestors(inode, |ancestor| {
            let entry = match self.try_get_inode(ancestor) {
                Some(e) => e,
                None => return,
            };

            match entry {
                Entry::Dir(dir) => {
                    if let Some((interest, _)) = dir.dynamic {
                        dynamics_required.push((DynamicInode::Inode(ancestor), interest));
                    }

                    if let Some(data) = dir.associated_data.as_ref() {
                        data.apply_to_interest(&mut interest)
                    }
                }
                Entry::File(f) => {
                    if let Some(data) = f.associated_data.as_ref() {
                        data.apply_to_interest(&mut interest)
                    }
                }
                _ => {}
            };
        });

        let mut phantom = None;
        if let Some(child_name) = looked_up_child {
            if let Some((phantom_fn, interest_fn, dyn_fn)) = self.inner.phantom_registry.get(&inode)
            {
                let child_name = child_name.to_string_lossy().into_owned();
                let phantom_ty = (phantom_fn)(&child_name);
                if let Some(phantom_ty) = phantom_ty {
                    trace!(
                        "looking up child {:?} under phantom inode {}",
                        child_name,
                        inode
                    );
                    let interest = (interest_fn)(phantom_ty);
                    dynamics_required.push((DynamicInode::Phantom(inode), interest));

                    phantom = Some(DynamicPhantom {
                        parent: inode,
                        child_name,
                        interest,
                        dir_associated_data: phantom_ty.into(),
                        dyn_fn: *dyn_fn,
                    });
                }
            }
        }

        let mut need_fetching = HashSet::new();
        for (inode, interest) in dynamics_required.iter() {
            let inode = match inode {
                DynamicInode::Inode(inode) | DynamicInode::Phantom(inode) => inode,
            };

            if let Some(state) = self.inner.dynamic_state.get(&(*inode, *interest)) {
                if state.time_collected.elapsed() <= STATE_TTL {
                    // cache is valid
                    continue;
                }
            }

            need_fetching.insert(*interest);
        }

        // apply interest
        for dynamic in need_fetching.iter() {
            match dynamic {
                DynamicStateType::EntityIds => {
                    interest.entities_by_id = true;
                }

                DynamicStateType::PlayerId => { /* always returned */ }
                &DynamicStateType::Block([x, y, z]) => {
                    interest.target_block = Some(BlockPos::new(x, y, z));
                }
            }
        }

        DynamicInterest {
            original_inode: inode,
            inodes: dynamics_required,
            need_fetching,
            interest,
            phantom,
        }
    }

    fn register_dynamic_entries(
        &mut self,
        dyn_fn: DynamicDirFn,
        parent: u64,
        interest: DynamicStateType,
        inodes: Option<InodeBlock>,
        state: &GameState,
        mut original_inode: Option<&mut u64>,
    ) {
        let mut registrationer =
            DynamicDirRegistrationer::new(inodes, parent, &mut self.inner.inode_alloc);
        (dyn_fn)(state, &mut registrationer);

        let (entries, inodes_used) = registrationer.take();

        // register dynamic entries
        for (new_inode, new_name, new_entry, new_parent) in entries {
            let new_parent = new_parent.unwrap_or(parent);
            self.inner
                .register(new_inode, new_entry, Some((new_parent, new_name)));
        }

        let state = DynamicState {
            inodes: inodes_used,
            time_collected: Instant::now(),
        };

        let prev = self.inner.dynamic_state.insert((parent, interest), state);
        if let Some(prev) = prev {
            for block in prev.inodes {
                for inode in block.iter_allocated() {
                    // TODO do for whole block at once
                    let root_child_name = self.inner.unregister(inode, parent);

                    match original_inode.as_mut() {
                        Some(orig) if **orig == inode => {
                            if let Some((orig_child_inode, _)) =
                                root_child_name.and_then(|orig_name| {
                                    self.inner.child_registry.get(&parent).and_then(|children| {
                                        children
                                            .iter()
                                            .find(|(_, child_name)| *child_name == orig_name)
                                    })
                                })
                            {
                                // remap
                                **orig = *orig_child_inode;
                                original_inode = None;
                            }
                        }
                        _ => {}
                    }
                }

                #[cfg(debug_assertions)]
                self.ensure_unused(&block);

                self.inner.inode_alloc.free(block);
            }
        }
    }

    #[cfg(debug_assertions)]
    fn ensure_unused(&self, block: &InodeBlock) {
        for inode in block.iter_all_without_allocating() {
            assert!(
                !self.inner.registry.contains_key(&inode),
                "inode {} is in registry",
                inode
            );
            assert!(
                !self.inner.child_registry.contains_key(&inode),
                "inode {} is in child registry",
                inode
            );
            assert!(
                !self.inner.parent_registry.contains_key(&inode),
                "inode {} is in parent registry",
                inode
            );
            for ty in DynamicStateType::iter() {
                assert!(
                    !self.inner.dynamic_state.contains_key(&(inode, ty)),
                    "inode {} is in dynamic state registry with ty {:?}",
                    inode,
                    ty
                );
            }
            assert!(
                !self.inner.phantom_registry.contains_key(&inode),
                "inode {} is in phantom registry",
                inode
            );

            if self.inode_remapping.contains_key(&inode) {
                warn!("removed inode {} is mapped", inode);
            }

            if let Some((k, v)) = self.inode_remapping.iter().find(|(_, v)| **v == inode) {
                warn!("removed inode {} is mapped to from {}", *v, *k);
            }
        }
    }

    pub fn ensure_generated(&mut self, state: &GameState, dynamics: DynamicInterest) {
        if let Some(phantom) = dynamics.phantom {
            let mut inodes = self.inner.inode_alloc.allocate();

            // make new phantom dir
            let phantom_dir = inodes.next().expect("no free inodes"); // should be at least 1
            self.inner.register(
                phantom_dir,
                DirEntry::build()
                    .associated_data(phantom.dir_associated_data)
                    .finish()
                    .into(),
                Some((phantom.parent, phantom.child_name)),
            );

            // register entries under new phantom dir
            self.register_dynamic_entries(
                phantom.dyn_fn,
                phantom_dir,
                phantom.interest,
                Some(inodes),
                state,
                None,
            );
        }

        let mut remapped_inode = dynamics.original_inode;
        for (inode, interest) in dynamics
            .inodes
            .into_iter()
            .filter_map(|(inode, ty)| match inode {
                DynamicInode::Inode(inode) if dynamics.need_fetching.contains(&ty) => {
                    Some((inode, ty))
                }
                _ => None,
            })
        {
            let dyn_fn = match self
                .lookup_inode(inode)
                .and_then(|e| e.as_dir())
                .and_then(|dir| dir.dynamic)
            {
                Some((int, dyn_fn)) if int == interest => dyn_fn,
                _ => {
                    log::warn!("inode {} is not a dynamic dir", inode);
                    continue;
                }
            };

            self.register_dynamic_entries(
                dyn_fn,
                inode,
                interest,
                None,
                state,
                if remapped_inode == dynamics.original_inode {
                    Some(&mut remapped_inode)
                } else {
                    None
                },
            );
        }

        if remapped_inode != dynamics.original_inode {
            self.inode_remapping
                .insert(dynamics.original_inode, remapped_inode);
            debug!(
                "mapping {} to new {}",
                dynamics.original_inode, remapped_inode
            );
        }
    }

    pub fn command_state_for_file(&self, file: u64) -> CommandState {
        let mut state = CommandState::default();

        self.walk_ancestors(file, |inode| {
            let associated_data = self.lookup_inode(inode).and_then(|e| match e {
                Entry::File(f) => f.associated_data.as_ref(),
                Entry::Dir(d) => d.associated_data.as_ref(),
                _ => None,
            });

            if let Some(data) = associated_data {
                data.apply_to_state(&mut state);
            }
        });

        state
    }

    fn walk_ancestors(&self, child: u64, mut per_parent: impl FnMut(u64)) {
        per_parent(child);

        let mut current = child;
        while let Some(parent) = self.inner.parent_registry.get(&current) {
            per_parent(*parent);
            current = *parent;
        }
    }

    pub fn maintain(&mut self) {
        const PERIOD: Duration = Duration::from_secs(30);
        if self.last_maintain.elapsed() < PERIOD {
            // dont bother too often
            return;
        }

        self.last_maintain = Instant::now();

        let prev = self.inode_remapping.len();
        self.inode_remapping
            .retain(|_, to| self.inner.registry.contains_key(to));
        let new = self.inode_remapping.len();
        if prev != new {
            trace!("removed stale inode mappings, from {} to {}", prev, new);
        }
    }
}

impl DynamicInterest {
    pub fn as_interest(&self) -> GameStateInterest {
        GameStateInterest { ..self.interest }
    }
}

impl FilesystemStructureBuilder {
    pub fn root(&self) -> u64 {
        self.inner.root
    }

    pub fn add_entry(&mut self, parent: u64, name: &'static str, entry: impl Into<Entry>) -> u64 {
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

    pub fn add_phantom(
        &mut self,
        inode: u64,
        parse_func: PhantomChildFn,
        interest_func: PhantomDynamicInterestFn,
        dyn_func: DynamicDirFn,
    ) {
        self.inner
            .phantom_registry
            .insert(inode, (parse_func, interest_func, dyn_func));
    }

    pub fn finish(self) -> FilesystemStructure {
        FilesystemStructure {
            inner: self.inner,
            inode_remapping: HashMap::new(),
            last_maintain: Instant::now(),
        }
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

    /// Returns child name under root parent, if it was under the given root
    fn unregister(&mut self, inode: u64, root_parent: u64) -> Option<Cow<'static, str>> {
        let mut frontier = vec![inode];
        trace!("start unregistering from {}", inode);
        while let Some(next) = frontier.pop() {
            trace!("removing inode {}", next);
            let _ = self.registry.remove(&next);
            let _ = self.phantom_registry.remove(&next);

            // try all interest types
            for ty in DynamicStateType::iter() {
                let _ = self.dynamic_state.remove(&(next, ty));
            }

            if let Some(children) = self.child_registry.remove(&next) {
                frontier.extend(children.iter().map(|(child, _)| *child));
                for (child, _) in children {
                    let parent = self.parent_registry.remove(&child);
                    assert!(
                        parent.is_some(),
                        "child {} was not registered in parent registry under parent {}",
                        child,
                        next
                    );
                }
            }

            let _ = self.parent_registry.remove(&inode);
        }

        let children = self
            .child_registry
            .get_mut(&root_parent)
            .expect("child is not registered under parent");

        let mut old_name = None;
        if let Some(idx) = children.iter().position(|(child, _)| *child == inode) {
            let (_, name) = children.remove(idx);
            old_name = Some(name)
        }

        old_name
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

    /// Overrides parent directory
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

    pub fn associated_data(mut self, data: EntryAssociatedData) -> Self {
        self.0.associated_data = Some(data);
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
            EntryAssociatedData::EntityId(id) => {
                if state.target_entity.is_none() {
                    state.target_entity = Some(TargetEntity::Entity(*id))
                }
            }
            EntryAssociatedData::PlayerId => {
                if state.target_entity.is_none() {
                    state.target_entity = Some(TargetEntity::Player)
                }
            }
            EntryAssociatedData::World(dim) => {
                if state.target_world.is_none() {
                    state.target_world = Some(*dim)
                }
            }
            EntryAssociatedData::Block(pos) => {
                if state.target_block.is_none() {
                    state.target_block = Some(*pos)
                }
            }
        }
    }

    fn apply_to_interest(&self, interest: &mut GameStateInterest) {
        match self {
            EntryAssociatedData::World(dim) if interest.target_world.is_none() => {
                interest.target_world = Some(*dim)
            }
            _ => {}
        }
    }
}

impl From<PhantomChildType> for EntryAssociatedData {
    fn from(ty: PhantomChildType) -> Self {
        match ty {
            PhantomChildType::Block([x, y, z]) => {
                EntryAssociatedData::Block(BlockPos::new(x, y, z))
            }
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

impl<'a> DynamicDirRegistrationer<'a> {
    pub fn new(
        starting_inodes: Option<InodeBlock>,
        parent: u64,
        inodes: &'a mut InodeBlockAllocator,
    ) -> Self {
        Self {
            entries: vec![],
            current_block: starting_inodes.unwrap_or_else(|| inodes.allocate()),
            all_blocks: smallvec![],
            inodes,
            parent,
        }
    }

    pub fn take(
        mut self,
    ) -> (
        Vec<(u64, Cow<'static, str>, Entry, Option<u64>)>,
        SmallVec<[InodeBlock; 2]>,
    ) {
        self.all_blocks.push(self.current_block);
        (self.entries, self.all_blocks)
    }

    pub fn add_root_entry(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        entry: impl Into<Entry>,
    ) -> u64 {
        self.add_entry_raw(None, name.into(), entry.into())
    }

    pub fn add_entry(
        &mut self,
        parent: u64,
        name: impl Into<Cow<'static, str>>,
        entry: impl Into<Entry>,
    ) -> u64 {
        self.add_entry_raw(Some(parent), name.into(), entry.into())
    }

    fn add_entry_raw(&mut self, parent: Option<u64>, name: Cow<'static, str>, entry: Entry) -> u64 {
        let inode = self.alloc_inode();
        self.entries.push((inode, name, entry, parent));
        inode
    }

    fn alloc_inode(&mut self) -> u64 {
        self.current_block.next().unwrap_or_else(|| {
            let new_block = self.inodes.allocate();
            let consumed = std::mem::replace(&mut self.current_block, new_block);
            self.all_blocks.push(consumed);
            self.current_block.next().unwrap() // not empty
        })
    }

    pub fn parent(&self) -> u64 {
        self.parent
    }
}
