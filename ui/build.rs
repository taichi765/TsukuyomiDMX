fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-dark".into())
        .with_bundled_translations("translations");
    #[cfg(debug_assertions)]
    let config = config.with_debug_info(true);
    slint_build::compile_with_config("slint/app-window.slint", config).expect("Slint build failed");
}
