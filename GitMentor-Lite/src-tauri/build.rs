fn main() {
    // 设置环境变量禁用libpng警告
    std::env::set_var("LIBPNG_NO_WARN", "1");

    tauri_build::build()
}
