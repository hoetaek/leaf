fn main() {
    // Expose the build target triple to the binary so `leaf update` can pick
    // the matching release asset at runtime. cargo sets TARGET for build
    // scripts; rustc-env makes it readable via env!("LEAF_TARGET").
    let target = std::env::var("TARGET").expect("cargo sets TARGET for build scripts");
    println!("cargo:rustc-env=LEAF_TARGET={target}");
}
