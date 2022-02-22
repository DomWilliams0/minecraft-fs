use std::error::Error;
use std::fmt::{Display, Formatter};
use std::process::exit;

use ipc::IpcChannel;

fn main() {
    env_logger::init();

    if let Err(err) = run() {
        println!(
            "error: {}\nusage: {} <mnt point>",
            err,
            std::env::args().next().as_deref().unwrap_or("mcfuse")
        );
        exit(1)
    }
}

#[derive(Debug)]
struct ArgError;

fn run() -> Result<(), Box<dyn Error>> {
    // get mnt point from args
    let mnt_point = std::env::args().nth(1).ok_or(ArgError)?;

    // connect to game
    let ipc = IpcChannel::open_existing()?;

    let opts = ["-o", "fsname=minecraft,rw"];
    let mnted = filesystem::mount(ipc, mnt_point.as_ref(), &opts)?;

    println!("mounted! ctrl c to exit");
    mnted.wait_for_unmount();
    println!("bye");

    Ok(())
}

impl Display for ArgError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing mount point arg")
    }
}

impl Error for ArgError {}
