# Releasing

Official releases are built by [the release workflow](../.github/workflows/release.yml).
A version tag builds every supported artifact, signs and notarizes the macOS
builds, generates checksums and provenance attestations, and publishes the
files to the tag's GitHub Release.

## Cut a release

1. Update `[package].version` in the root `Cargo.toml` and commit the change.
2. Create and push a matching bare `YEAR.MONTH.PATCH` tag:

   ```bash
   git tag 2026.7.4
   git push origin 2026.7.4
   ```

The workflow rejects malformed tags and tags that differ from the Cargo
version. To rehearse without publishing, run `Actions → Release → Run workflow`
and choose `all`, `linux`, `freebsd`, `windows`, or `macos`. Manual artifacts
expire after one day.

## Published artifacts

| Runner | Artifacts |
| --- | --- |
| `ubuntu-latest` | `linux-gnu-{x86_64,i686,armv7,riscv64}`, `linux-musl-{x86_64,i686,armv7,riscv64}`, and `freebsd-{x86_64,aarch64}` |
| `ubuntu-24.04-arm` | `linux-{gnu,musl}-aarch64` |
| `windows-latest` | `windows-msvc-x86_64` |
| `windows-11-arm` | `windows-msvc-aarch64` |
| `macos-latest` | `apple-darwin-{x86_64,aarch64}` |

GNU builds use cargo-zigbuild with a glibc 2.17 floor (2.27 for riscv64, the
first glibc with that port). Musl builds are statically linked. All target
jobs download the same frontend build, fonts, and npm license snapshot; each
target adds its own Rust dependency licenses.

The FreeBSD slices are also cross-compiled with cargo-zigbuild — zig ships the
FreeBSD 14+ libc stubs, which sets the FreeBSD 14 floor. aarch64 FreeBSD is a
Tier 3 Rust target with no prebuilt standard library, so that slice builds std
from source on a nightly toolchain date-pinned in the workflow and in
`Makefile.toml` (`FREEBSD_NIGHTLY`). riscv64 builds use vendored PUC Lua 5.1
instead of LuaJIT, which has no mainline riscv64 backend; KOReader sidecars
are plain Lua 5.1 data chunks, so the semantics are identical.

The workflow rejects GNU binaries with symbols newer than their glibc floor
and musl binaries with a dynamic interpreter or shared-library dependency.
macOS builds pin and verify macOS 10.12 for Intel and macOS 11 for Apple
Silicon. Every runnable artifact also gets a `--version` smoke test, with
`LD_BIND_NOW=1` for Linux GNU builds so the complete symbol chain must
resolve. The binaries run natively, including 32-bit x86 and Intel macOS under
Rosetta, except for ARMv7 and riscv64, which run under qemu-user. FreeBSD
binaries cannot run on the Linux runners (qemu-user has no FreeBSD mode), so
those slices stop at static checks: machine type, the
`/libexec/ld-elf.so.1` dynamic linker path, and the FreeBSD ELF branding
note.

The local cargo-make configuration also retains Windows GNU and universal
macOS packages for convenience. Those are not GitHub Release artifacts.

## Local release prerequisites

`cargo make release` cross-compiles every Linux and Windows artifact locally
and needs a cross toolchain on `PATH` for each of them, using exactly the
binary names configured in `Makefile.toml` and `.cargo/config.toml`:

| Prefix | Source |
| --- | --- |
| `x86_64-linux-gnu-`, `aarch64-linux-gnu-` | Homebrew glibc cross toolchains |
| `x86_64-unknown-linux-musl-`, `aarch64-unknown-linux-musl-` | musl cross toolchains |
| `i686-unknown-linux-gnu-`, `i686-unknown-linux-musl-` | 32-bit x86 cross toolchains |
| `armv7-unknown-linux-gnueabihf-`, `armv7-unknown-linux-musleabihf-` | 32-bit ARM cross toolchains |
| `x86_64-w64-mingw32-` | `mingw-w64` |

The i686 and ARMv7 toolchains became required when the 32-bit Linux targets
were added; without them `cargo make release` aborts with "linker … not
found". Each target also needs its Rust standard library
(`rustup target add <target>`).

The riscv64 and FreeBSD tasks need no cross gcc: they build with
`cargo-zigbuild`, which needs `zig` 0.16 and `cargo-zigbuild` on `PATH`. The
aarch64 FreeBSD task additionally needs the pinned nightly with its source
component:

```bash
rustup toolchain install nightly-2026-07-15 --component rust-src
```

## Protected release environment

Create a protected GitHub environment named `release`, require reviewer
approval, and store these environment secrets:

| Secret | Purpose |
| --- | --- |
| `APPLE_DEVELOPER_ID` | Full Developer ID Application identity, including Team ID |
| `APPLE_USERNAME` | Apple ID used by notarytool |
| `APPLE_PASSWORD` | App-specific Apple ID password |
| `APPLE_CERT_P12_BASE64` | Base64-encoded Developer ID certificate export |
| `APPLE_CERT_PASSWORD` | Password for the `.p12` export |

The workflow imports the certificate into a temporary keychain and stores the
notarization credentials in a temporary `koshelf-notary` keychain profile. The
Apple ID and password are never written to `.env`.

For local signed releases, copy `.env.example` to `.env`, select a profile
name, and create that profile once:

```bash
xcrun notarytool store-credentials <name> --apple-id <apple-id> --team-id <team-id>
cargo make release
```

Protect version tags with a `*.*.*` tag ruleset that restricts tag creation,
movement, and deletion to repository administrators.

## Verification

Every release must contain exactly 16 ZIPs plus `SHA256SUMS`. Verify a
download with the checksum file and its GitHub provenance attestation:

```bash
sha256sum --check SHA256SUMS
gh attestation verify linux-gnu-x86_64.zip --repo paviro/KoShelf
```

The workflow additionally checks that every runnable binary starts, along
with the GNU symbol-version ceiling, static musl linkage, the FreeBSD ABI
checks, minimum macOS versions, macOS signatures, and the exact artifact
filename set before publish.
