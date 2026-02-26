use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use autocxx_build::Builder as AutocxxBuilder;
use syntheon as syn;

const OGDF_TAG: &str = "foxglove-202510";
const PERF_FLAGS: &[&str] = &["-O3", "-DNDEBUG", "-g0", "-fomit-frame-pointer"];
const NATIVE_CPU_FLAGS: &[&str] = &["-march=native", "-mtune=native"];

struct OgdfLayout {
    _cache_lock: syn::LockedCacheDir,
    archive_path: PathBuf,
    source_dir: PathBuf,
    build_dir: PathBuf,
    install_dir: PathBuf,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");

    let ogdf_layout = ogdf_layout();
    println!(
        "cargo:rerun-if-changed={}",
        ogdf_layout.archive_path.display()
    );
    let install_dir = ensure_ogdf(&ogdf_layout);

    let cpp_include = PathBuf::from("cpp/include");
    let ogdf_include = install_dir.join("include");
    let ogdf_release_include = install_dir.join("include/ogdf-release");

    let mut build = AutocxxBuilder::new(
        "src/lib.rs",
        [
            cpp_include.as_path(),
            ogdf_include.as_path(),
            ogdf_release_include.as_path(),
        ],
    )
    .extra_clang_args(
        &clang_args()
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>(),
    )
    .build()
    .expect("autocxx code generation failed");

    build
        .file("cpp/src/spqr.cpp")
        .file("cpp/src/mps.cpp")
        .flag("-std=c++17")
        .include(&cpp_include)
        .include(&ogdf_include)
        .include(&ogdf_release_include);

    apply_perf_flags(&mut build);
    build.compile("graphum_ffi");

    for lib_dir in lib_dirs(&install_dir) {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    }
    println!("cargo:rustc-link-lib=static=OGDF");
    println!("cargo:rustc-link-lib=static=COIN");
    if env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("unix") {
        println!("cargo:rustc-link-lib=pthread");
    }

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cpp/include/types.hpp");
    println!("cargo:rerun-if-changed=cpp/include/mps.hpp");
    println!("cargo:rerun-if-changed=cpp/include/spqr.hpp");
    println!("cargo:rerun-if-changed=cpp/src/mps.cpp");
    println!("cargo:rerun-if-changed=cpp/src/spqr.cpp");
}

fn clang_args() -> Vec<String> {
    let mut args = vec!["-std=c++17".to_string()];
    for dir in syn::clang_system_include_dirs() {
        args.push("-isystem".to_string());
        args.push(dir.display().to_string());
    }
    args
}

fn ensure_ogdf_source(layout: &OgdfLayout) -> PathBuf {
    if layout.source_dir.join("CMakeLists.txt").exists() {
        return layout.source_dir.clone();
    }

    if let Some(parent) = layout.source_dir.parent() {
        fs::create_dir_all(parent).expect("failed to create ogdf source parent directory");
    }
    let root = layout
        .source_dir
        .parent()
        .unwrap_or_else(|| panic!("missing parent for {}", layout.source_dir.display()));
    syn::extract_tar_gz(&layout.archive_path, root);
    if layout.source_dir.join("CMakeLists.txt").exists() {
        return layout.source_dir.clone();
    }
    panic!(
        "OGDF source tree not found under {} after extraction",
        layout.source_dir.display()
    );
}

fn ensure_ogdf(layout: &OgdfLayout) -> PathBuf {
    if !available_lib_dirs(&layout.install_dir).is_empty() {
        return layout.install_dir.clone();
    }
    ensure_ogdf_source(layout);
    build_ogdf(layout)
}

fn build_ogdf(layout: &OgdfLayout) -> PathBuf {
    if !available_lib_dirs(&layout.install_dir).is_empty() {
        return layout.install_dir.clone();
    }

    let cmake = syn::CmakeRunner::new(&layout.build_dir);
    let jobs = cmake.jobs();
    let capabilities = cmake.capabilities();
    if jobs > 1 && !capabilities.build_parallel {
        println!(
            "cargo:warning=cmake --build --parallel requires cmake 3.12+; building with a single job"
        );
    }
    if jobs > 1 && !capabilities.install_parallel {
        println!(
            "cargo:warning=cmake --install --parallel requires cmake 3.31+; installing with a single job"
        );
    }

    fs::create_dir_all(&layout.build_dir).expect("failed to create ogdf build directory");
    let perf_flags = perf_flag_string();
    let mut configure = Command::new("cmake");
    configure
        .arg("-S")
        .arg(&layout.source_dir)
        .arg("-B")
        .arg(&layout.build_dir)
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg(format!("-DCMAKE_CXX_FLAGS={perf_flags}"))
        .arg(format!("-DCMAKE_C_FLAGS={perf_flags}"))
        .arg(format!("-DCMAKE_CXX_FLAGS_RELEASE={perf_flags}"))
        .arg(format!("-DCMAKE_C_FLAGS_RELEASE={perf_flags}"))
        .arg("-DBUILD_SHARED_LIBS=OFF")
        .arg("-DOGDF_WARNING_ERRORS=OFF")
        .arg("-DOGDF_INCLUDE_CGAL=OFF")
        .arg("-DOGDF_DEBUG_MODE=NONE")
        .arg("-DCMAKE_POSITION_INDEPENDENT_CODE=ON")
        .arg(format!(
            "-DCMAKE_INSTALL_PREFIX={}",
            layout.install_dir.display()
        ));
    syn::run(&mut configure, "cmake configuration failed");

    cmake.build_target("OGDF", Some("Release"), "cmake build failed");
    cmake.install(Some("Release"), "cmake install failed");

    if available_lib_dirs(&layout.install_dir).is_empty() {
        panic!(
            "OGDF build did not produce a usable library under {}",
            layout.install_dir.display()
        );
    }
    layout.install_dir.clone()
}

fn ogdf_layout() -> OgdfLayout {
    let tag = OGDF_TAG.to_string();
    let archive_path = syn::vendor_dir().join(format!("ogdf-{tag}.tar.gz"));
    if !archive_path.is_file() {
        panic!(
            "missing vendored OGDF archive at {}",
            archive_path.display()
        );
    }

    let fingerprint = syn::CacheFingerprint::builder()
        .flag("native", syn::wants_native_cpu_flags())
        .build();
    let cache = syn::cache_dir(tag.as_str())
        .with_fingerprint_opt(fingerprint)
        .lock();
    let root = cache.path().to_path_buf();

    OgdfLayout {
        _cache_lock: cache,
        archive_path,
        source_dir: root.join(format!("ogdf-{tag}")),
        build_dir: root.join("build"),
        install_dir: root.join("install"),
    }
}

fn available_lib_dirs(root: &Path) -> Vec<PathBuf> {
    ["lib", "lib64"]
        .into_iter()
        .map(|dir| root.join(dir))
        .filter(|dir| dir.join("libOGDF.a").exists() && dir.join("libCOIN.a").exists())
        .collect()
}

fn lib_dirs(root: &Path) -> Vec<PathBuf> {
    let dirs = available_lib_dirs(root);
    if dirs.is_empty() {
        panic!(
            "OGDF libraries missing under {} (looked for lib/ and lib64/)",
            root.display()
        );
    }
    dirs
}

fn perf_flag_string() -> String {
    PERF_FLAGS
        .iter()
        .copied()
        .chain(native_cpu_flags().iter().copied())
        .collect::<Vec<_>>()
        .join(" ")
}

fn apply_perf_flags(build: &mut cc::Build) {
    for flag in PERF_FLAGS {
        build.flag(flag);
    }
    if syn::wants_native_cpu_flags() {
        for flag in NATIVE_CPU_FLAGS {
            build.flag(flag);
        }
    }
}

fn native_cpu_flags() -> &'static [&'static str] {
    if syn::wants_native_cpu_flags() {
        NATIVE_CPU_FLAGS
    } else {
        &[]
    }
}
