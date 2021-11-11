use std::path::Path;

mod filesystem;
mod mount;

fn main() {
    let mnt_point = Path::new("./mnt");
    let opts = ["-o", "fsname=minecraft,rw"];
    let mnted = mount::mount(mnt_point, &opts).expect("mount failed");

    println!("ctrl c to exit");
    mnted.wait_for_unmount();
    println!("bye");
}
