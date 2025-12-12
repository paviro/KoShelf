Although there are many ways to use this tool here is how I use it:

Syncthing Sync: I use Syncthing to sync both my books folder and KoReader settings folder from my e-reader to my server
Books and Statistics: I point to the synced books folder with --books-path and to statistics.sqlite3 in the synced KoReader settings folder with --statistics-db
Web Server Mode: I then run KoShelf in web server mode (without --output) - it will automatically rebuild when files change
Nginx Reverse Proxy: I use an nginx reverse proxy for HTTPS and to restrict access
My actual setup:

# My server command - runs continuously with file watching and statistics
./koshelf --books-path ~/syncthing/Books \
         --statistics-db ~/syncthing/KOReaderSettings/statistics.sqlite3 \
         --port 3000
This way, every time Syncthing pulls updates from my e-reader, the website automatically updates with my latest reading progress, new highlights, and updated statistics.

Current Documentation Gaps

Missing Step-by-Step Setup Guide
• No clear progression from "install Syncthing" to "working sync"
• Device pairing process not documented
• Folder configuration lacks specific screenshots/examples

KOReader Plugin Setup Unclear
• Plugin installation instructions are minimal
• No troubleshooting for plugin issues
• Configuration within KOReader not covered

Network Configuration Missing
• Local network discovery requirements not explained
• Firewall/router settings not mentioned
• Device IP configuration guidance absent

Troubleshooting Section Incomplete
• No sync conflict resolution guidance
• Connection failure debugging missing
• Platform-specific issues (macOS, Linux) not addressed
