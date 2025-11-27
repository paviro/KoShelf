use super::*;
use crate::partial_md5::calculate_partial_md5;
use mlua::Lua;
use regex::Regex;

#[derive(Debug)]
struct PartialMd5Params {
    step: i32,
    size: i32,
    loop_start: i32,
    loop_end: i32,
}

fn parse_partial_md5_params(util_source: &str, util_path: &Path) -> PartialMd5Params {
    let step_size_re =
        Regex::new(r"local\s+step\s*,\s*size\s*=\s*(\d+)\s*,\s*(\d+)").expect("bad regex");
    let step_caps = step_size_re.captures(util_source).unwrap_or_else(|| {
        panic!(
            "Unable to locate partialMD5 step/size in {}",
            util_path.display()
        )
    });
    let step: i32 = step_caps[1]
        .parse()
        .expect("Failed to parse partialMD5 step");
    let size: i32 = step_caps[2]
        .parse()
        .expect("Failed to parse partialMD5 size");

    let loop_re = Regex::new(
        r#"(?s)for\s+i\s*=\s*([-\d]+)\s*,\s*([-\d]+)\s*do.*?file:seek\(\s*"set"\s*,\s*(?:bit\.)?lshift\(\s*step\s*,\s*2\s*\*\s*i\s*\)\s*\)"#,
    )
    .expect("bad regex");
    let loop_caps = loop_re.captures(util_source).unwrap_or_else(|| {
        panic!(
            "Unable to locate partialMD5 sampling loop in {}",
            util_path.display()
        )
    });
    let loop_start: i32 = loop_caps[1]
        .parse()
        .expect("Failed to parse sampling loop start");
    let loop_end: i32 = loop_caps[2]
        .parse()
        .expect("Failed to parse sampling loop end");

    PartialMd5Params {
        step,
        size,
        loop_start,
        loop_end,
    }
}

/// Load KoReader's actual util.lua and extract the sampling logic.
/// We modify it slightly to return raw samples (so we can hash in Rust)
/// since mlua doesn't have KoReader's md5 library.
fn get_koreader_samples(lua: &Lua, koreader_path: &Path, filepath: &Path) -> Vec<u8> {
    let util_lua_path = koreader_path.join("frontend/util.lua");
    let util_lua = std::fs::read_to_string(&util_lua_path).unwrap_or_else(|_| {
        panic!(
            "Failed to read KoReader's util.lua at {}",
            util_lua_path.display()
        )
    });

    let params = parse_partial_md5_params(&util_lua, &util_lua_path);

    // We need to:
    // 1. Mock the dependencies that util.lua requires
    // 2. Extract just the partialMD5 sampling logic and return raw samples
    let lua_code = format!(
        r#"
        local bit = require("bit")

        -- Load KoReader's util.lua to extract the partialMD5 function
        -- We'll parse it to find the function definition
        local util_source = [==[{util_lua}]==]

        -- Extract and adapt the partialMD5 function to return samples instead of hash
        -- KoReader's partialMD5 does:
        --   for i = -1, 10 do
        --       file:seek("set", lshift(step, 2*i))
        --       local sample = file:read(size)
        --       ...
        --   end
        local function getSamples(filepath)
            if not filepath then return "" end
            local file = io.open(filepath, "rb")
            if not file then return "" end

            local step, size = {step}, {size}
            local samples = {{}}

            -- This is the exact loop from KoReader's partialMD5 in util.lua
            for i = {loop_start}, {loop_end} do
                file:seek("set", bit.lshift(step, 2*i))
                local sample = file:read(size)
                if sample then
                    table.insert(samples, sample)
                else
                    break
                end
            end
            file:close()

            return table.concat(samples)
        end

        return getSamples
        "#,
        step = params.step,
        size = params.size,
        loop_start = params.loop_start,
        loop_end = params.loop_end
    );

    let get_samples_fn: mlua::Function = lua
        .load(&lua_code)
        .eval()
        .unwrap_or_else(|e| panic!("Failed to load KoReader sampling function: {}", e));

    let filepath_str = filepath.to_string_lossy().to_string();
    let samples: mlua::String = get_samples_fn
        .call(filepath_str)
        .expect("Failed to get samples from KoReader Lua");

    samples.as_bytes().to_vec()
}

/// Hash samples using Rust's MD5 (same as our implementation)
fn hash_samples(samples: &[u8]) -> String {
    let digest = md5::compute(samples);
    format!("{:x}", digest)
}

#[test]
fn test_partial_md5_pdf() {
    let koreader_dir = get_koreader_dir();
    let test_file = koreader_dir.path().join("spec/front/unit/data/tall.pdf");

    if !test_file.exists() {
        panic!(
            "Test file not found at {}. The spec/front symlink may not be set up correctly.",
            test_file.display()
        );
    }

    let lua = Lua::new();

    // Get samples using KoReader's Lua logic
    let koreader_samples = get_koreader_samples(&lua, koreader_dir.path(), &test_file);
    let koreader_hash = hash_samples(&koreader_samples);

    // Calculate hash with our Rust implementation
    let rust_hash = calculate_partial_md5(&test_file).expect("Failed to calculate partial MD5");

    eprintln!("File: {}", test_file.display());
    eprintln!("KoReader samples: {} bytes", koreader_samples.len());
    eprintln!("KoReader hash:    {}", koreader_hash);
    eprintln!("Rust hash:        {}", rust_hash);

    assert_eq!(
        rust_hash, koreader_hash,
        "Rust partialMD5 should match KoReader's Lua implementation for PDF"
    );
}

#[test]
fn test_partial_md5_epub() {
    let koreader_dir = get_koreader_dir();
    let test_file = koreader_dir.path().join("spec/front/unit/data/leaves.epub");

    if !test_file.exists() {
        panic!(
            "Test file not found at {}. The spec/front symlink may not be set up correctly.",
            test_file.display()
        );
    }

    let lua = Lua::new();

    // Get samples using KoReader's Lua logic
    let koreader_samples = get_koreader_samples(&lua, koreader_dir.path(), &test_file);
    let koreader_hash = hash_samples(&koreader_samples);

    // Calculate hash with our Rust implementation
    let rust_hash = calculate_partial_md5(&test_file).expect("Failed to calculate partial MD5");

    eprintln!("File: {}", test_file.display());
    eprintln!("KoReader samples: {} bytes", koreader_samples.len());
    eprintln!("KoReader hash:    {}", koreader_hash);
    eprintln!("Rust hash:        {}", rust_hash);

    assert_eq!(
        rust_hash, koreader_hash,
        "Rust partialMD5 should match KoReader's Lua implementation for EPUB"
    );
}

