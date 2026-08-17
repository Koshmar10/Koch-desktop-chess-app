fn main() {
    // `include_dir!` embeds migrations/ at compile time; without this, editing a .sql
    // file wouldn't trigger a rebuild since no .rs file changed.
    println!("cargo:rerun-if-changed=src/db/migrations/");
    tauri_build::build()
}
