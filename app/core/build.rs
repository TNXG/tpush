fn main() {
    println!("cargo:rerun-if-changed=ffi/tpush_core.udl");
    uniffi::generate_scaffolding("ffi/tpush_core.udl")
        .expect("failed to generate UniFFI scaffolding");
}
