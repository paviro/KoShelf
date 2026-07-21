# Syncthing Setups

There are many different ways to configure Syncthing with KoShelf depending on your specific devices and infrastructure. This directory contains example setups shared by the community to help you get started.

> **Note:** If you read on multiple devices, syncing a single `statistics.sqlite3` with Syncthing requires strictly serial reading — SQLite files cannot be merged at the file level, so concurrent writes on two devices lose data. See [Reading Statistics from Multiple Devices](../koreader-setup.md#reading-statistics-from-multiple-devices) for alternatives (KOReader's built-in statistics sync, or supplying each device's database to KoShelf separately).

## Available Setups

- [Setup from alva](setup_from_alva_seal.md): A complete guide for setting up Syncthing on a server/PC alongside KoShelf using Docker, including steps for pairing with a Kobo e-reader.
