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
and choose `all`, `linux`, `windows`, or `macos`. Manual artifacts expire after
one day.

## Published artifacts

| Runner | Artifacts |
| --- | --- |
| `ubuntu-latest` | `linux-gnu-{x86_64,aarch64}` and `linux-musl-x86_64` |
| `ubuntu-24.04-arm` | `linux-musl-aarch64` |
| `windows-latest` | `windows-msvc-x86_64` |
| `windows-11-arm` | `windows-msvc-aarch64` |
| `macos-latest` | `apple-darwin-{x86_64,aarch64}` |

GNU builds use cargo-zigbuild with a glibc 2.17 floor. Musl builds are
statically linked. All target jobs download the same frontend build, fonts,
and npm license snapshot; each target adds its own Rust dependency licenses.

The local cargo-make configuration also retains Windows GNU and universal
macOS packages for convenience. Those are not GitHub Release artifacts.

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

Every release must contain exactly eight ZIPs plus `SHA256SUMS`. Verify a
download with the checksum file and its GitHub provenance attestation:

```bash
sha256sum --check SHA256SUMS
gh attestation verify linux-gnu-x86_64.zip --repo paviro/KoShelf
```

The workflow additionally checks the GNU symbol-version ceiling, static musl
linkage, macOS signatures, and the exact artifact filename set before publish.
