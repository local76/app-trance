fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_resource::compile_for("assets/brand/ssm.rc", &["ssm"], embed_resource::NONE);
        embed_resource::compile_for("assets/brand/install.rc", &["install"], embed_resource::NONE);
        embed_resource::compile_for("assets/brand/uninstall.rc", &["uninstall"], embed_resource::NONE);
    }
}
