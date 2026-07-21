use std::path::Path;
use std::process::Command;

/// Build a `Command` invoking npm with the given args.
///
/// On Windows `npm` is a `npm.cmd` shim that `Command::new("npm")` can't spawn
/// directly, so we go through `cmd /C npm`. This keys off the build host
/// (`cfg!(windows)` in a build script reflects the host, not the target), so a
/// unix host cross-compiling to `x86_64-pc-windows-gnu` still uses a plain `npm`.
fn npm_command(args: &[&str]) -> Command {
    let mut cmd = if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg("npm");
        cmd
    } else {
        Command::new("npm")
    };
    cmd.args(args);
    cmd
}

/// Build the React+Vite frontend used by the runtime and static export paths.
pub(crate) fn compile_react_frontend(skip_npm_install: bool, skip_react_build: bool) {
    let frontend_dir = Path::new("frontend");
    let frontend_package = frontend_dir.join("package.json");

    if !frontend_package.exists() {
        eprintln!("No frontend/package.json found, skipping React frontend build");
        return;
    }

    if skip_react_build {
        eprintln!("Skipping React frontend build (KOSHELF_SKIP_REACT_BUILD=1)");
        return;
    }

    let frontend_lock = frontend_dir.join("package-lock.json");
    let frontend_node_modules = frontend_dir.join("node_modules");

    let should_install = !frontend_node_modules.exists()
        || (frontend_lock.exists()
            && frontend_node_modules
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                < frontend_lock
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH));

    if should_install {
        if skip_npm_install {
            panic!(
                "frontend/node_modules missing/outdated but npm install is disabled (KOSHELF_SKIP_NPM_INSTALL). \
                 Run `npm --prefix frontend ci`/`npm --prefix frontend install`, or unset the env var."
            );
        }

        eprintln!("Installing frontend npm dependencies...");
        let install_subcommand = if frontend_lock.exists() {
            "ci"
        } else {
            "install"
        };

        let install_output = npm_command(&["--prefix", "frontend", install_subcommand])
            .output()
            .expect("Failed to install frontend npm dependencies.");
        if !install_output.status.success() {
            panic!(
                "frontend npm install failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&install_output.stdout),
                String::from_utf8_lossy(&install_output.stderr)
            );
        }
    }

    eprintln!("Building React frontend with Vite...");
    let build_output = npm_command(&["--prefix", "frontend", "run", "build"])
        .output()
        .expect("Failed to run frontend build.");

    if !build_output.status.success() {
        panic!(
            "React frontend build failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build_output.stdout),
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    let dist_dir = frontend_dir.join("dist");
    if !dist_dir.exists() {
        panic!("frontend build completed but dist directory was not found");
    }
    eprintln!("React frontend build completed: {}", dist_dir.display());
}
