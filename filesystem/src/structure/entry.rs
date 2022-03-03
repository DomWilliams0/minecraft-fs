use std::any::{Any, TypeId};
use std::borrow::Cow;

use ipc::generated::{BlockPos, Dimension};
use ipc::{CommandState, TargetEntity};

use crate::state::{GameState, GameStateInterest};
use crate::structure::registry::{DynamicDirRegistrationer, DynamicStateType, PhantomChildType};
use crate::structure::{EntryFilterResult, FileBehaviour};

pub type DynamicDirFn = fn(&GameState, &mut DynamicDirRegistrationer);
pub type PhantomDynamicInterestFn = fn(PhantomChildType) -> DynamicStateType;
pub type FileFilterFn = fn(&GameState) -> bool;
pub type DirFilterFn = fn(&GameState) -> EntryFilterResult;
pub type LinkTargetFn = Box<dyn Fn(&GameState) -> Option<Cow<'static, str>> + Send>;

#[derive(PartialEq)]
pub enum Entry {
    File(FileEntry),
    Dir(DirEntry),
    Link(LinkEntry),
}

#[derive(Default)]
pub struct DirEntry {
    dynamic: Option<(DynamicStateType, DynamicDirFn)>,
    associated_data: Option<EntryAssociatedData>,
    filter: Option<DirFilterFn>,
}

pub struct FileEntry {
    behaviour: FileBehaviour,
    associated_data: Option<EntryAssociatedData>,
    filter: Option<FileFilterFn>,
}

pub struct LinkEntry {
    target: LinkTargetFn,
    target_typeid: TypeId,
    filter: Option<FileFilterFn>,
}

#[derive(Default)]
pub struct DirEntryBuilder(DirEntry);

pub struct FileEntryBuilder(FileEntry);

pub struct LinkEntryBuilder(LinkEntry);

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum EntryAssociatedData {
    PlayerId,
    EntityId(i32),
    World(Dimension),
    Block(BlockPos),
}

impl Entry {
    pub fn as_dir(&self) -> Option<&DirEntry> {
        match self {
            Entry::Dir(dir) => Some(dir),
            _ => None,
        }
    }
}

impl FileEntryBuilder {
    /// Overrides parent directory
    #[allow(dead_code)]
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
    pub fn build(behaviour: FileBehaviour) -> FileEntryBuilder {
        FileEntryBuilder(FileEntry {
            behaviour,
            associated_data: None,
            filter: None,
        })
    }

    pub fn behaviour(&self) -> &FileBehaviour {
        &self.behaviour
    }

    pub fn associated_data(&self) -> Option<EntryAssociatedData> {
        self.associated_data
    }
}

impl LinkEntryBuilder {
    fn new(target: LinkTargetFn, target_typeid: TypeId) -> Self {
        Self(LinkEntry {
            target,
            target_typeid,
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
    pub fn build(
        target: impl Fn(&GameState) -> Option<Cow<'static, str>> + Send + 'static,
    ) -> LinkEntryBuilder {
        let typeid = target.type_id();
        LinkEntryBuilder::new(Box::new(target), typeid)
    }

    pub fn target(&self) -> &LinkTargetFn {
        &self.target
    }
}

impl DirEntry {
    pub fn build() -> DirEntryBuilder {
        DirEntryBuilder::default()
    }

    pub fn dynamic(&self) -> Option<(DynamicStateType, DynamicDirFn)> {
        self.dynamic
    }

    pub fn associated_data(&self) -> Option<EntryAssociatedData> {
        self.associated_data
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
    pub fn apply_to_state(&self, state: &mut CommandState) {
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

    pub fn apply_to_interest(&self, interest: &mut GameStateInterest) {
        match self {
            EntryAssociatedData::World(dim) => {
                if interest.target_world.is_none() {
                    interest.target_world = Some(*dim)
                }
            }
            EntryAssociatedData::Block(pos) => {
                if interest.target_block.is_none() {
                    interest.target_block = Some(*pos)
                }
            }

            EntryAssociatedData::PlayerId => {}
            EntryAssociatedData::EntityId(_) => {}
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

mod entry_impls {
    use std::fmt::{Debug, Formatter};

    use crate::structure::entry::{DirEntry, Entry, FileEntry, LinkEntry};

    macro_rules! cmp_fn_ptrs {
        ($a:expr, $b:expr) => {
            match ($a, $b) {
                (Some(a), Some(b)) => std::ptr::eq(a as *const (), b as *const ()),
                (None, None) => true,
                _ => false,
            }
        };
    }

    impl PartialEq for FileEntry {
        fn eq(&self, other: &Self) -> bool {
            self.behaviour == other.behaviour
                && self.associated_data == other.associated_data
                && cmp_fn_ptrs!(self.filter, other.filter)
        }
    }

    impl PartialEq for DirEntry {
        fn eq(&self, other: &Self) -> bool {
            self.associated_data == other.associated_data
                && cmp_fn_ptrs!(self.filter, other.filter)
                && match (self.dynamic, other.dynamic) {
                    (Some((ty_a, fn_a)), Some((ty_b, fn_b))) => {
                        ty_a == ty_b && std::ptr::eq(fn_a as *const (), fn_b as *const ())
                    }
                    (None, None) => true,
                    _ => false,
                }
        }
    }

    impl PartialEq for LinkEntry {
        fn eq(&self, other: &Self) -> bool {
            self.target_typeid == other.target_typeid && cmp_fn_ptrs!(self.filter, other.filter)
        }
    }

    macro_rules! debug_fn {
        ($opt_fn:expr) => {
            $opt_fn.map(|func| func as *const ())
        };
    }

    impl Debug for FileEntry {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("File")
                .field("behaviour", &self.behaviour)
                .field("associated_data", &self.associated_data)
                .field("filter", &debug_fn!(self.filter))
                .finish()
        }
    }

    impl Debug for DirEntry {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Dir")
                .field(
                    "dynamic",
                    &self
                        .dynamic
                        .as_ref()
                        .map(|(ty, func)| (ty, *func as *const ())),
                )
                .field("associated_data", &self.associated_data)
                .field("filter", &debug_fn!(self.filter))
                .finish()
        }
    }

    impl Debug for LinkEntry {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Link")
                .field("target", &(Box::as_ref(&self.target) as *const _))
                .field("filter", &debug_fn!(self.filter))
                .finish()
        }
    }

    impl Debug for Entry {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Entry::File(e) => Debug::fmt(e, f),
                Entry::Dir(e) => Debug::fmt(e, f),
                Entry::Link(_) => write!(f, "Link(..)"),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use ipc::generated::CommandType;
        use ipc::BodyType;

        use crate::structure::entry::{EntryAssociatedData, FileFilterFn};
        use crate::structure::registry::DynamicStateType;
        use crate::structure::FileBehaviour;

        use super::*;

        #[test]
        fn fn_comparison() {
            fn a(_: i32) -> i32 {
                2
            }
            fn b(_: i32) -> i32 {
                5
            }

            type MyFn = fn(i32) -> i32;
            let a1_ = a as MyFn;
            let a2_ = a as MyFn;
            let b_ = b as MyFn;

            assert!(cmp_fn_ptrs!(Some(a1_), Some(a1_)));
            assert!(cmp_fn_ptrs!(Some(a1_), Some(a2_)));
            assert!(!cmp_fn_ptrs!(Some(a1_), Some(b_)));

            assert!(!cmp_fn_ptrs!(Some(a1_), Option::<MyFn>::None));
            assert!(cmp_fn_ptrs!(Option::<MyFn>::None, Option::<MyFn>::None));
        }

        #[test]
        fn file_entry_comparison() {
            let a = FileEntry::build(FileBehaviour::ReadWrite(
                CommandType::EntityHealth,
                BodyType::Float,
            ))
            .finish();
            let b = FileEntry::build(FileBehaviour::ReadWrite(
                CommandType::EntityHealth,
                BodyType::Float,
            ))
            .finish();
            assert_eq!(a, b);
        }

        #[test]
        fn dir_entry_comparison() {
            let mk_dir = || {
                DirEntry::build()
                    .associated_data(EntryAssociatedData::PlayerId)
                    .dynamic(DynamicStateType::PlayerId, |_, _| eprintln!("wowee"))
                    .finish()
            };

            let a = mk_dir();
            let b = mk_dir();

            assert_eq!(a, b);
        }

        #[test]
        fn link_entry_comparison() {
            let captured = 50i32;

            let mk_link1 = || LinkEntry::build(move |_| Some(captured.to_string().into())).finish();
            let mk_link2 = || LinkEntry::build(|_| None).finish();

            let mk_link_with_filter = |filter: Option<FileFilterFn>| {
                let mut l = LinkEntry::build(|_| Some("teehee".to_string().into()));
                if let Some(filter) = filter {
                    l = l.filter(filter);
                }
                l.finish()
            };

            let a = mk_link1();
            let b = mk_link1();
            let c = mk_link2();
            let d = mk_link2();

            assert_eq!(a, b);
            assert_eq!(b, a);
            assert_eq!(c, d);

            assert_ne!(a, c);
            assert_ne!(b, c);

            assert_ne!(
                mk_link_with_filter(None),
                mk_link_with_filter(Some(|_| true)),
            );

            assert_eq!(mk_link_with_filter(None), mk_link_with_filter(None),);
        }
    }
}
