fn main() {
    // tauri-build 默认不跟踪 icons/ 目录，图标变更后 Cargo 不会重新编译
    println!("cargo:rerun-if-changed=icons");
    tauri_build::build()
}
