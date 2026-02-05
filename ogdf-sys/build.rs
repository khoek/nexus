use std::env;
use std::fs::{self, File};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::Command;

use autocxx_build::Builder as AutocxxBuilder;
use flate2::read::GzDecoder;
use tar::Archive;

const OGDF_TAG: &str = "foxglove-202510";
const PERF_FLAGS: &[&str] = &["-O3", "-DNDEBUG", "-g0", "-fomit-frame-pointer"];
const NATIVE_CPU_FLAGS: &[&str] = &["-march=native", "-mtune=native"];

fn vendor_dir() -> PathBuf {
    PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be provided by cargo"),
    )
    .join("vendor")
}

struct OgdfLayout {
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
    for dir in system_include_dirs() {
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
    extract_archive(&layout.archive_path, root);
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
    let jobs = parallel_jobs();
    if !available_lib_dirs(&layout.install_dir).is_empty() {
        return layout.install_dir.clone();
    }
    let cmake = cmake_capabilities();
    if jobs > 1 && !cmake.build_parallel {
        println!(
            "cargo:warning=cmake --build --parallel requires cmake 3.12+; building with a single job"
        );
    }
    if jobs > 1 && !cmake.install_parallel {
        println!(
            "cargo:warning=cmake --install --parallel requires cmake 3.31+; installing with a single job"
        );
    }

    fs::create_dir_all(&layout.build_dir).expect("failed to create ogdf build directory");
    let perf_flags = perf_flag_string();
    run(
        Command::new("cmake")
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
            )),
        "cmake configuration failed",
    );

    let mut build_cmd = Command::new("cmake");
    build_cmd
        .arg("--build")
        .arg(&layout.build_dir)
        .arg("--target")
        .arg("OGDF")
        .arg("--config")
        .arg("Release");
    apply_parallel(&mut build_cmd, jobs, cmake.build_parallel);
    run(&mut build_cmd, "cmake build failed");

    let mut install_cmd = Command::new("cmake");
    install_cmd
        .arg("--install")
        .arg(&layout.build_dir)
        .arg("--config")
        .arg("Release");
    if cmake.install_parallel && jobs > 1 {
        let jobs_str = jobs.to_string();
        install_cmd
            .arg("--parallel")
            .arg(&jobs_str)
            .env("CMAKE_INSTALL_PARALLEL_LEVEL", jobs_str);
    }
    run(&mut install_cmd, "cmake install failed");

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
    let archive_path = vendor_dir().join(format!("ogdf-{tag}.tar.gz"));
    if !archive_path.is_file() {
        panic!(
            "missing vendored OGDF archive at {}",
            archive_path.display()
        );
    }
    let cache_root = ogdf_cache_root();
    let dir_key = sanitize_component(&tag);
    let root = cache_root.join(dir_key);
    OgdfLayout {
        archive_path,
        source_dir: root.join(format!("ogdf-{tag}")),
        build_dir: root.join("build"),
        install_dir: root.join("install"),
    }
}

fn ogdf_cache_root() -> PathBuf {
    let target_root = target_root();
    let pkg = sanitize_component(
        &env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME must be provided by cargo"),
    );
    let target_triple = sanitize_component(
        &env::var("TARGET").expect("TARGET must be provided by cargo for build scripts"),
    );
    target_root.join("build-deps").join(pkg).join(target_triple)
}

fn target_root() -> PathBuf {
    if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        return PathBuf::from(dir);
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be provided by cargo"));
    if let Some(target_dir) = out_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|name| name == "target"))
    {
        return target_dir.to_path_buf();
    }
    PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be provided by cargo"),
    )
    .join("target")
}

fn sanitize_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

fn system_include_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for flag in ["-print-file-name=include", "-print-file-name=include-fixed"] {
        let path = gcc_include_path(flag);
        if path.exists() {
            dirs.push(path);
        }
    }
    if dirs.is_empty() {
        panic!(
            "failed to locate system include directories via gcc; install clang headers or a \
             working gcc toolchain"
        );
    }
    dirs
}

fn gcc_include_path(flag: &str) -> PathBuf {
    let output = Command::new("gcc")
        .arg(flag)
        .output()
        .unwrap_or_else(|e| panic!("failed to invoke gcc {flag}: {e}"));
    if !output.status.success() {
        panic!("gcc {flag} exited with {}", output.status);
    }
    let path = String::from_utf8_lossy(&output.stdout);
    let trimmed = path.trim();
    if trimmed.is_empty() {
        panic!("gcc {flag} returned an empty include path");
    }
    PathBuf::from(trimmed)
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

#[derive(Clone, Copy, Debug)]
struct CmakeCapabilities {
    build_parallel: bool,
    install_parallel: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CmakeVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl CmakeVersion {
    const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    fn at_least(self, other: CmakeVersion) -> bool {
        (self.major, self.minor, self.patch) >= (other.major, other.minor, other.patch)
    }
}

fn parallel_jobs() -> usize {
    env::var("NUM_JOBS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .and_then(NonZeroUsize::new)
        .map(NonZeroUsize::get)
        .or_else(|| {
            std::thread::available_parallelism()
                .ok()
                .map(NonZeroUsize::get)
        })
        .unwrap_or(1)
}

fn cmake_capabilities() -> CmakeCapabilities {
    let version = cmake_version();
    CmakeCapabilities {
        build_parallel: version.at_least(CmakeVersion::new(3, 12, 0)),
        install_parallel: version.at_least(CmakeVersion::new(3, 31, 0)),
    }
}

fn cmake_version() -> CmakeVersion {
    let output = Command::new("cmake")
        .arg("--version")
        .output()
        .unwrap_or_else(|e| panic!("failed to query cmake version: {e}"));
    if !output.status.success() {
        panic!("cmake --version exited with {}", output.status);
    }
    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_else(|e| panic!("cmake --version output not valid UTF-8: {e}"));
    let version_line = stdout
        .lines()
        .find(|line| line.starts_with("cmake version"))
        .unwrap_or_else(|| panic!("unexpected cmake --version output: {stdout}"));
    let mut tokens = version_line.split_whitespace();
    tokens.next();
    tokens.next();
    let raw_version = tokens
        .next()
        .unwrap_or_else(|| panic!("failed to locate cmake version in {version_line}"));
    parse_cmake_version(raw_version)
}

fn parse_cmake_version(raw: &str) -> CmakeVersion {
    let mut parts = raw.split('.');
    let major = parse_version_component(parts.next(), raw, "major");
    let minor = parse_version_component(parts.next(), raw, "minor");
    let patch = parts
        .next()
        .map(|part| parse_version_component(Some(part), raw, "patch"))
        .unwrap_or(0);
    CmakeVersion::new(major, minor, patch)
}

fn parse_version_component(part: Option<&str>, raw: &str, label: &str) -> u32 {
    let raw_part = part.unwrap_or_else(|| {
        panic!("cmake version missing {label} component in {raw}");
    });
    let digits: String = raw_part
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        panic!("cmake version missing {label} digits in {raw}");
    }
    digits
        .parse::<u32>()
        .unwrap_or_else(|e| panic!("failed to parse cmake version {label} from {raw}: {e}"))
}

fn apply_parallel(cmd: &mut Command, jobs: usize, supports_parallel: bool) {
    if jobs <= 1 {
        return;
    }
    let jobs_str = jobs.to_string();
    if supports_parallel {
        cmd.arg("--parallel").arg(&jobs_str);
    }
    cmd.env("CMAKE_BUILD_PARALLEL_LEVEL", jobs_str);
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
    if wants_native_cpu_flags() {
        for flag in NATIVE_CPU_FLAGS {
            build.flag(flag);
        }
    }
}

fn native_cpu_flags() -> &'static [&'static str] {
    if wants_native_cpu_flags() {
        NATIVE_CPU_FLAGS
    } else {
        &[]
    }
}

fn wants_native_cpu_flags() -> bool {
    let Ok(flags) = env::var("CARGO_ENCODED_RUSTFLAGS") else {
        return false;
    };

    let mut last_target_cpu = None;
    let mut saw_dash_c = false;

    for token in flags.split('\u{1f}') {
        if saw_dash_c {
            if let Some(cpu) = token.strip_prefix("target-cpu=") {
                last_target_cpu = Some(cpu);
            }
            saw_dash_c = false;
        }

        if token == "-C" {
            saw_dash_c = true;
            continue;
        }

        if let Some(cpu) = token.strip_prefix("-Ctarget-cpu=") {
            last_target_cpu = Some(cpu);
            continue;
        }
        if let Some(cpu) = token.strip_prefix("target-cpu=") {
            last_target_cpu = Some(cpu);
        }
    }

    last_target_cpu == Some("native")
}

fn extract_archive(archive_path: &Path, out_dir: &Path) {
    let file = File::open(archive_path)
        .unwrap_or_else(|e| panic!("failed to open {}: {e}", archive_path.display()));
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);
    archive.unpack(out_dir).unwrap_or_else(|e| {
        panic!(
            "failed to extract archive {} into {}: {e}",
            archive_path.display(),
            out_dir.display()
        )
    });
}

fn run(cmd: &mut Command, err: &str) {
    let status = cmd.status().unwrap_or_else(|e| panic!("{err}: {e}"));
    if !status.success() {
        panic!("{err}: status {status}");
    }
}
