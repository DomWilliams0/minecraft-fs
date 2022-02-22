pub use entry::{DirEntry, Entry, EntryAssociatedData, FileEntry};
pub use registry::{EntryFilterResult, FileBehaviour, FilesystemStructure};
pub use structure::create_structure;

mod entry;
mod inode;
mod registry;
#[allow(clippy::module_inception)]
mod structure;

pub(crate) mod fatptr {

    #[derive(Debug)]
    #[repr(C)]
    pub struct SplitFatPtr {
        data: *const (),
        vtable: *const (),
    }

    impl SplitFatPtr {
        pub unsafe fn split<T: ?Sized>(ptr: *const T) -> SplitFatPtr {
            let ptr_ref: *const *const T = &ptr;
            let decomp_ref = ptr_ref as *const [usize; 2];
            std::mem::transmute(*decomp_ref)
        }
    }

    impl PartialEq for SplitFatPtr {
        fn eq(&self, other: &Self) -> bool {
            std::ptr::eq(self.vtable, other.vtable)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn safety() {
            type MyFn = fn(&i32) -> i32;
            type MyFnDyn = dyn Fn() -> i32;
            type MyFnBoxed = Box<dyn Fn() -> i32>;

            let fn_no_data1: MyFn = |_| 2;
            let fn_no_data2: MyFn = |_| 5;

            let dummy = 5;
            let fn_data1: MyFnBoxed = Box::new(move || dummy);
            let fn_data2: MyFnBoxed = Box::new(move || dummy + 2);

            unsafe {
                assert_eq!(
                    SplitFatPtr::split(fn_no_data1 as *const MyFn),
                    SplitFatPtr::split(fn_no_data1 as *const MyFn)
                );
                assert_eq!(
                    SplitFatPtr::split(fn_no_data2 as *const MyFn),
                    SplitFatPtr::split(fn_no_data2 as *const MyFn)
                );
                assert_eq!(
                    SplitFatPtr::split(&fn_data1 as *const MyFnDyn),
                    SplitFatPtr::split(&fn_data1 as *const MyFnDyn),
                );
                assert_eq!(
                    SplitFatPtr::split(&fn_data2 as *const MyFnDyn),
                    SplitFatPtr::split(&fn_data2 as *const MyFnDyn),
                );
            }
        }
    }
}
