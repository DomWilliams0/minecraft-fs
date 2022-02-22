use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fmt::Debug;
use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

use log::*;
use smallvec::SmallVec;
use strum::{EnumIter, IntoEnumIterator};

use ipc::generated::{BlockPos, CommandType};
use ipc::{BodyType, CommandState};

use crate::state::{GameState, GameStateInterest};
use crate::structure::entry::{
    DirEntry, DynamicDirFn, Entry, EntryAssociatedData, PhantomDynamicInterestFn,
};
use crate::structure::fatptr::SplitFatPtr;
use crate::structure::inode::InodeBlockAllocator;

pub struct FilesystemStructure {
    inner: StructureInner,
}

pub struct FilesystemStructureBuilder {
    inner: StructureInner,
}

struct StructureInner {
    inode_alloc: InodeBlockAllocator,
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
    /// (inode, its parent)
    inodes: HashSet<(u64, u64)>,
    time_collected: Instant,
}

pub struct DynamicDirRegistrationer<'a> {
    /// (inode, name, entry, parent)
    new_entries: Vec<(u64, Cow<'static, str>, Entry, u64)>,
    /// (reused inode, its parent)
    to_retain: HashSet<(u64, u64)>,
    structure: &'a mut FilesystemStructure,
    parent: u64,
}

#[derive(Copy, Clone)]
pub enum EntryFilterResult {
    IncludeSelf,
    IncludeAllChildren,
    Exclude,
}

#[derive(derivative::Derivative)]
#[derivative(PartialEq, Debug)]
pub enum FileBehaviour {
    ReadOnly(CommandType, BodyType),
    WriteOnly(CommandType, BodyType),
    ReadWrite(CommandType, BodyType),
    // TODO rw with different types
    Static(Cow<'static, str>),
    /// Not readable or writable
    ForShow,
    CommandProxy {
        readme: Cow<'static, str>,
        #[derivative(Debug = "ignore")]
        #[derivative(PartialEq(compare_with = "cmp_cmd_proxy_fn"))]
        produce_cmd_fn: CommandProxyFn,
    },
}

type CommandProxyFn = Box<dyn Fn(&str) -> Option<String> + Send>;

pub struct DynamicInterest {
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
        let mut registry = HashMap::with_capacity(256);

        // register root
        let root = inode_alloc.allocate_static();
        registry.insert(root, Entry::Dir(DirEntry::default()));

        FilesystemStructureBuilder {
            inner: StructureInner {
                inode_alloc,
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
        self.inner.registry.get(&inode)
    }

    fn try_get_inode(&self, inode: u64) -> Option<&Entry> {
        self.inner.registry.get(&inode)
    }

    fn get_inode(&self, inode: u64) -> &Entry {
        self.try_get_inode(inode)
            .unwrap_or_else(|| panic!("unregistered inode {}", inode,))
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

    pub fn lookup_children(
        &self,
        inode: u64,
    ) -> Option<impl Iterator<Item = (&Entry, &str)> + ExactSizeIterator + '_> {
        self.inner.child_registry.get(&inode).map(|v| {
            v.iter()
                .map(|(inode, name)| (self.get_inode(*inode), name.as_ref()))
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
                    if let Some((interest, _)) = dir.dynamic() {
                        dynamics_required.push((DynamicInode::Inode(ancestor), interest));
                    }

                    if let Some(data) = dir.associated_data() {
                        data.apply_to_interest(&mut interest)
                    }
                }
                Entry::File(f) => {
                    if let Some(data) = f.associated_data() {
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
        state: &GameState,
    ) {
        let mut registrationer = DynamicDirRegistrationer::new(parent, self);
        (dyn_fn)(state, &mut registrationer);

        let (new_entries, to_retain) = registrationer.take_entries();

        let new_inodes = new_entries
            .iter()
            .map(|(ino, _, _, parent)| (*ino, *parent))
            .collect::<HashSet<_>>();

        for (new_inode, new_name, new_entry, new_parent) in new_entries.into_iter() {
            let parent_info = Some((new_parent, new_name));
            self.inner.register(new_inode, new_entry, parent_info);
        }

        let new_state = DynamicState {
            inodes: new_inodes.clone(),
            time_collected: Instant::now(),
        };

        let mut old_inodes = HashSet::new();
        if let Some(prev_state) = self
            .inner
            .dynamic_state
            .insert((parent, interest), new_state)
        {
            let empty = std::mem::replace(&mut old_inodes, prev_state.inodes);
            debug_assert!(empty.is_empty());
            std::mem::forget(empty);

            old_inodes.retain(|key| !to_retain.contains(key));
        }

        let retained_dirs = to_retain
            .iter()
            .map(|(_, parent)| *parent)
            .collect::<HashSet<_>>();

        for retained_dir in retained_dirs {
            trace!(
                "looking in retained dir {} to remove stale entries",
                retained_dir
            );
            let children = self
                .inner
                .child_registry
                .get(&retained_dir)
                .expect("dir was not retained");
            for (child_ino, child_name) in children {
                let key = (*child_ino, retained_dir);
                if !InodeBlockAllocator::is_static(*child_ino)
                    && !to_retain.contains(&key)
                    && !new_inodes.contains(&key)
                {
                    trace!(
                        "removing stale entry {:?} inode {} under {}",
                        child_name,
                        child_ino,
                        retained_dir
                    );

                    old_inodes.insert((*child_ino, parent));
                }
            }
        }

        for (old_ino, _) in old_inodes {
            self.inner.unregister(old_ino, parent);

            #[cfg(debug_assertions)]
            self.ensure_unused(old_ino);
        }
    }

    #[cfg(debug_assertions)]
    fn ensure_unused(&self, inode: u64) {
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
    }

    #[cfg(debug_assertions)]
    fn ensure_valid_state(&self) {
        let mut current_ino = None;
        let mut current_parent = None;

        let res = std::panic::catch_unwind(AssertUnwindSafe(|| {
            for (ino, _) in self.inner.registry.iter() {
                current_ino = Some(*ino);
                current_parent = None;

                // 1 is root
                if *ino != 1 {
                    let parent = self
                        .inner
                        .parent_registry
                        .get(ino)
                        .expect("missing registration of parent");
                    current_parent = Some(*parent);
                    let parents_children = self
                        .inner
                        .child_registry
                        .get(parent)
                        .expect("parent is missing child registration");

                    parents_children
                        .iter()
                        .find(|(child_ino, _)| *child_ino == *ino)
                        .expect("child is not in parent's children");
                }
            }

            current_parent = None;

            for inode in self.inner.phantom_registry.keys() {
                current_ino = Some(*inode);
                assert!(
                    self.inner.registry.contains_key(inode),
                    "phantom inode is not registered"
                )
            }

            for (inode, _) in self.inner.dynamic_state.keys() {
                current_ino = Some(*inode);
                assert!(
                    self.inner.registry.contains_key(inode),
                    "dynamic inode is not registered"
                )
            }
        }));

        if res.is_err() {
            panic!(
                "assertion panic: inode={:?}, parent={:?}",
                current_ino, current_parent
            )
        }
    }

    pub fn ensure_generated(&mut self, state: &GameState, dynamics: DynamicInterest) {
        if let Some(phantom) = dynamics.phantom {
            // make new phantom dir
            let phantom_dir = self.inner.inode_alloc.allocate();
            self.inner.register(
                phantom_dir,
                DirEntry::build()
                    .associated_data(phantom.dir_associated_data)
                    .finish()
                    .into(),
                Some((phantom.parent, phantom.child_name)),
            );

            // register entries under new phantom dir
            self.register_dynamic_entries(phantom.dyn_fn, phantom_dir, phantom.interest, state);
        }

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
                .and_then(|dir| dir.dynamic())
            {
                Some((int, dyn_fn)) if int == interest => dyn_fn,
                _ => {
                    log::warn!("inode {} is not a dynamic dir", inode);
                    continue;
                }
            };

            self.register_dynamic_entries(dyn_fn, inode, interest, state);
        }

        #[cfg(debug_assertions)]
        self.ensure_valid_state()
    }

    pub fn command_state_for_file(&self, file: u64) -> CommandState {
        let mut state = CommandState::default();

        self.walk_ancestors(file, |inode| {
            let associated_data = self.lookup_inode(inode).and_then(|e| match e {
                Entry::File(f) => f.associated_data(),
                Entry::Dir(d) => d.associated_data(),
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
        let inode = self.inner.inode_alloc.allocate_static();

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

    fn unregister(&mut self, inode: u64, root_parent: u64) {
        // TODO might not need to recurse
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
                    let _ = self.parent_registry.remove(&child);
                }
            }

            let _ = self.parent_registry.remove(&inode);
        }

        let children = self
            .child_registry
            .get_mut(&root_parent)
            .expect("child is not registered under parent");

        if let Some(idx) = children.iter().position(|(child, _)| *child == inode) {
            let _ = children.remove(idx);
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

impl<'a> DynamicDirRegistrationer<'a> {
    fn new(parent: u64, structure: &'a mut FilesystemStructure) -> Self {
        Self {
            new_entries: vec![],
            to_retain: HashSet::new(),
            structure,
            parent,
        }
    }

    /// (vec of (new inode, its name under the parent, the entry, its parent),
    /// set of (reused inode, its parent))
    pub fn take_entries(
        self,
    ) -> (
        Vec<(u64, Cow<'static, str>, Entry, u64)>,
        HashSet<(u64, u64)>,
    ) {
        (self.new_entries, self.to_retain)
    }

    pub fn add_root_entry(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        entry: impl Into<Entry>,
    ) -> u64 {
        self.add_entry_raw(self.parent, name.into(), entry.into())
    }

    pub fn add_entry(
        &mut self,
        parent: u64,
        name: impl Into<Cow<'static, str>>,
        entry: impl Into<Entry>,
    ) -> u64 {
        self.add_entry_raw(parent, name.into(), entry.into())
    }

    fn add_entry_raw(&mut self, parent: u64, name: Cow<'static, str>, entry: Entry) -> u64 {
        trace!("adding raw dynamic entry under {}: {:?}", parent, name);

        // TODO put unwrap into try_get_inode again
        match self.structure.lookup_child(parent, name.as_ref().as_ref()) {
            Some((prev_inode, prev_entry)) if &entry == prev_entry => {
                // identical entry, reuse inode
                trace!("identical entry found at {}! reusing it", prev_inode);
                self.to_retain.insert((prev_inode, parent));
                prev_inode
            }
            _ => {
                let inode = self.structure.inner.inode_alloc.allocate();
                trace!("new inode: {}", inode);
                self.new_entries.push((inode, name, entry, parent));
                inode
            }
        }
    }

    pub fn parent(&self) -> u64 {
        self.parent
    }
}

fn cmp_cmd_proxy_fn(a: &CommandProxyFn, b: &CommandProxyFn) -> bool {
    let (a, b) = unsafe {
        (
            SplitFatPtr::split(a as *const CommandProxyFn),
            SplitFatPtr::split(b as *const CommandProxyFn),
        )
    };
    a == b
}
