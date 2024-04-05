fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("windows") {
        embed_resource::compile("assets/icon.rc", embed_resource::NONE);
    }
}