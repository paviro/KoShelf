# Installation

## Home Assistant

Using Home Assistant? Install KoShelf as an add-on with just one click below.

[![Open your Home Assistant instance and show the dashboard of an add-on.](https://my.home-assistant.io/badges/supervisor_addon.svg)](https://my.home-assistant.io/redirect/supervisor_addon/?addon=5d189d71_koshelf&repository_url=https%3A%2F%2Fgithub.com%2Fpaviro%2Fhome-assistant-addons)

## Docker Compose

Deploy KoShelf using the community-maintained Docker image.

### Quick Start

1. Create a `docker-compose.yml` file:

```yaml
services:
    koshelf:
        image: ghcr.io/devtigro/koshelf:latest
        ports:
            - '3000:3000'
        volumes:
            - /path/to/your/books:/books:ro
            - /path/to/your/settings:/settings:ro
        restart: unless-stopped
```

2. Update the volume paths:

- Replace `/path/to/your/books` with the absolute path to your book library
- Replace `/path/to/your/settings` with the absolute path to your settings directory

3. Start the container:

```bash
docker compose up -d
```

4. Access KoShelf at http://localhost:3000

Docker Image Repository: [koshelf-docker](https://github.com/DevTigro/koshelf-docker)

## Prebuilt Binaries

Download a prebuilt binary from the [releases page](https://github.com/paviro/koshelf/releases). Binaries are available for:

- Windows (x64)
- macOS (Apple Silicon, Intel & Universal)
- Linux (x64 and ARM64)

KoShelf is a command line tool — you need to run it from a terminal (macOS/Linux) or PowerShell/Command Prompt (Windows). Double-clicking the executable won't work since it requires command line arguments.

**Note for Windows users**: Windows Defender will likely flag and delete the Windows binary as a virus (more information [here](https://medium.com/opsops/is-windows-broken-7f8de8b8f3ad)). This is a false positive if you downloaded the binary directly from this repo. To use the binary:

1. Restore it from Windows Defender's protection history (Windows Security > Virus & threat protection > Protection history > Restore)
2. Launch the binary from PowerShell or Windows Terminal with arguments - double-clicking will cause it to close immediately since no arguments are provided

### First Time Using Command Line?

If you've never used a command line before, here's how to get started:

**Windows:**

1. Press `Win + R`, type `powershell`, and press Enter
2. Navigate to where you downloaded the KoShelf binary (e.g., `cd C:\Users\YourName\Downloads`)
3. Run the tool with your desired arguments (see [Configuration](configuration.md#examples))

**macOS and Linux:**

1. Press `Cmd + Space`, type `terminal`, and press Enter
2. Navigate to where you downloaded the KoShelf binary (e.g., `cd ~/Downloads`)
3. Make the file executable: `chmod +x koshelf` (should not be needed on macOS as the binary is signed)
4. Run the tool with your desired arguments (see [Configuration](configuration.md#examples))

**Example:**

```bash
# Navigate to your downloads folder
cd ~/Downloads  # macOS/Linux
cd C:\Users\YourName\Downloads  # Windows

# Run KoShelf with your books folder
./koshelf export ./my-library-site --library-path /path/to/your/library
```

**Tip:** On most terminals, you can drag and drop the downloaded binary file into the terminal window to insert its full path.

### System-Wide Installation (Linux/macOS)

If you plan to use KoShelf frequently, you can move the binary to `/usr/local/bin/` to make it available system-wide:

```bash
# Move the binary to system PATH (requires sudo)
sudo mv koshelf /usr/local/bin/

# Now you can run it from anywhere
koshelf export ~/my-library-site --library-path ~/Books
```

## From Source

### Prerequisites

- Rust 1.70+ (for building)
- Node.js and npm (React frontend build pipeline)

### Building the tool

```bash
git clone https://github.com/paviro/KoShelf
cd KoShelf

# Build the Rust binary
cargo build --release
```

The binary will be available at `target/release/koshelf`.

**Note:** The React frontend is built during `cargo build` and embedded into the binary.
