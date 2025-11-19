fn main() {
    // Check if we are compiling in release mode
    if std::env::var("PROFILE").unwrap_or_default() == "release" {
        println!("cargo:warning=Optimized for performance!");
    }
}