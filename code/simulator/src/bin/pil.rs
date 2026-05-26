// PIL binary — simulator over USB transport.
//
// Transport: PostcardClient::try_new_raw_nusb (USB bulk transfer).
// Everything else (physics, TUI, scripted) is identical to the host binary.
//
// Status: stub — USB transport setup not yet implemented (M2.2 scope is HOST only).
// Tracked in code/simulator/spec.md §11.

fn main() {
    eprintln!("simulator-pil: USB transport not yet implemented (see spec.md §11)");
    std::process::exit(1);
}
