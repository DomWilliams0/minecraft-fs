use ipc::{IpcChannel, Response};
use std::path::Path;

fn main() {
    env_logger::init();

    // connect to game
    let ipc = IpcChannel::open_existing().expect("failed to connect to game");

    let mnt_point = Path::new("./mnt");
    let opts = ["-o", "fsname=minecraft,rw"];
    let mnted = filesystem::mount(ipc, mnt_point, &opts).expect("mount failed");

    println!("ctrl c to exit");
    mnted.wait_for_unmount();
    println!("bye");
}
