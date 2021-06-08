use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

fn find_hyperscan() -> Result<PathBuf> {
    cargo_emit::rerun_if_env_changed!("HYPERSCAN_ROOT");

    let link_kind = if cfg!(feature = "static") { "static" } else { "dylib" };

    if let Ok(prefix) = env::var("HYPERSCAN_ROOT") {
        let prefix = Path::new(&prefix);
        if !prefix.exists() || !prefix.is_dir() {
            bail!("HYPERSCAN_ROOT should point to a directory that exists.");
        }

        let inc_path = prefix.join("include/hs");
        let link_path = prefix.join("lib");
        if link_path.exists() && link_path.is_dir() {
            cargo_emit::rustc_link_search!(link_path.to_string_lossy() => "native");
        } else {
            bail!("`$HYPERSCAN_ROOT/lib` subdirectory not found.");
        }

        let mut link_libs = vec![];

        if !cfg!(feature = "compile") && cfg!(feature = "runtime") {
            link_libs.push("static=hs_runtime".into());
        } else {
            link_libs.push(format!("{}=hs", link_kind));

            if cfg!(feature = "static") {
                link_libs.push("c++".into());
            }
        }

        if cfg!(feature = "chimera") {
            link_libs.push("chimera".into());
            link_libs.push("pcre".into());
        }

        cargo_emit::warning!(
            "building with Hyperscan with {} library @ {:?}, libs={:?}, link_paths=[{:?}], include_paths=[{:?}]",
            link_kind,
            prefix,
            link_libs,
            link_path,
            inc_path
        );

        for lib in link_libs {
            cargo_emit::rustc_link_lib!(lib);
        }

        Ok(inc_path)
    } else {
        let libhs = pkg_config::Config::new()
            .statik(cfg!(feature = "static"))
            .cargo_metadata(true)
            .env_metadata(true)
            .probe("libhs")?;

        cargo_emit::warning!(
            "building with Hyperscan {} with {} library, libs={:?}, link_paths={:?}, include_paths={:?}",
            libhs.version,
            link_kind,
            libhs.libs,
            libhs.link_paths,
            libhs.include_paths
        );

        if cfg!(feature = "chimera") {
            let libch = pkg_config::Config::new()
                .statik(cfg!(feature = "static"))
                .cargo_metadata(true)
                .env_metadata(true)
                .probe("libch")?;

            cargo_emit::warning!(
                "building with Chimera {} with {} library, libs={:?}, link_paths={:?}, include_paths={:?}",
                libch.version,
                link_kind,
                libch.libs,
                libch.link_paths,
                libch.include_paths
            );
        }

        libhs
            .include_paths
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("missing include path"))
    }
}

#[cfg(any(feature = "gen", not(target_pointer_width = "64")))]
fn generate_binding(inc_dir: &Path, out_dir: &Path) -> Result<()> {
    let out_file = out_dir.join("hyperscan.rs");
    let inc_file = inc_dir.join("hs.h");
    let inc_file = inc_file.to_str().expect("header file");

    cargo_emit::warning!("generating raw Hyperscan binding file @ {}", out_file.display());

    cargo_emit::rerun_if_changed!(inc_file);

    bindgen::builder()
        .header(inc_file)
        .use_core()
        .ctypes_prefix("::libc")
        .clang_args(&["-x", "c++", "-std=c++11"])
        .allowlist_var("^HS_.*")
        .allowlist_type("^hs_.*")
        .allowlist_function("^hs_.*")
        .blocklist_type("^__darwin_.*")
        .size_t_is_usize(true)
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .generate()
        .map_err(|_| anyhow::Error::msg("generate binding files"))?
        .write_to_file(out_file)
        .with_context(|| "write wrapper")
}

#[cfg(all(not(feature = "gen"), target_pointer_width = "64"))]
fn generate_binding(_: &Path, out_dir: &Path) -> Result<()> {
    std::fs::copy("src/hyperscan.rs", out_dir.join("hyperscan.rs"))
        .map(|_| ())
        .with_context(|| "copy binding file")
}

#[cfg(all(feature = "chimera", any(feature = "gen", not(target_pointer_width = "64"))))]
fn generate_chimera_binding(inc_dir: &Path, out_dir: &Path) -> Result<()> {
    let out_file = out_dir.join("chimera.rs");
    let inc_file = inc_dir.join("ch.h");
    let inc_file = inc_file.to_str().expect("header file");

    cargo_emit::warning!("generating raw Chimera binding file @ {}", out_file.display());

    cargo_emit::rerun_if_changed!(inc_file);

    bindgen::builder()
        .header(inc_file)
        .use_core()
        .ctypes_prefix("::libc")
        .clang_args(&["-x", "c++", "-std=c++11"])
        .allowlist_var("^CH_.*")
        .allowlist_type("^ch_.*")
        .allowlist_function("^ch_.*")
        .blocklist_type("^__darwin_.*")
        .size_t_is_usize(true)
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .generate()
        .map_err(|_| anyhow::Error::msg("generate binding files"))?
        .write_to_file(out_file)
        .with_context(|| "write wrapper")
}

#[cfg(all(not(feature = "gen"), feature = "chimera", target_pointer_width = "64"))]
fn generate_chimera_binding(_: &Path, out_dir: &Path) -> Result<()> {
    std::fs::copy("src/chimera.rs", out_dir.join("chimera.rs"))
        .map(|_| ())
        .with_context(|| "copy binding file")
}

#[cfg(all(not(feature = "chimera"), target_pointer_width = "64"))]
fn generate_chimera_binding(_: &Path, _: &Path) -> Result<()> {
    Ok(())
}

fn main() -> Result<()> {
    let inc_dir = find_hyperscan()
        .with_context(|| anyhow!("please download and install hyperscan from https://www.hyperscan.io/"))?;
    let out_dir = env::var("OUT_DIR")?;
    let out_dir = Path::new(&out_dir);

    generate_binding(&inc_dir, &out_dir)?;

    if cfg!(feature = "chimera") {
        generate_chimera_binding(&inc_dir, &out_dir)?;
    }

    Ok(())
}
