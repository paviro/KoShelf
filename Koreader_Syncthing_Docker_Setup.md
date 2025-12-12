# Koreader Syncthing Docker Setup

Although there are many ways to use this tool here is how one of them:

Syncthing Sync: I use Syncthing to sync both my books folder and KoReader settings folder from my Kobo to my server
Books and Statistics: I point to the synced books folder with --books-path and to statistics.sqlite3 in the synced KoReader settings folder with --statistics-db
Use of Docker image: [KoShelf Docker](https://github.com/DevTigro/koshelf-docker)


## Syncthing

### On the server/pc where KoShelf is running

running the [syncthing](https://docs.linuxserver.io/images/docker-syncthing/) docker image 

setting of the path for the volumes to the needs of the user

```
---
services:
  syncthing:
    image: linuxserver/syncthing:latest
    container_name: syncthing
    hostname: syncthing #optional
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
      - WUD_TRIGGER_DOCKER_LOCAL_PRUNE=true
    volumes:
      - /opt/syncthing/config:/config
      - /opt/syncthing/koreader:/koreader
      - /opt/syncthing/kobobooks:/books
    ports:
      - 22000:22000/tcp
      - 22000:22000/udp
      - 21027:21027/udp
      - 8384:8384
    restart: unless-stopped
```  

### Syncthing for Koreader 

This describes the steps on Kobo devices (path for other devises may need adaption)

It is assumed that Koreader is already installed and running.

- download the synching [plugin](https://github.com/jasonchoimtt/koreader-syncthing/archive/refs/heads/main.zip)
- Attach the Kobo to a device via usb
- go to the hidden folder .adds on the E-reader
- got to the sub folder koreader 
- got to the sub folder plugins
- put the downloaded zip file in the plugins folder
- unzip the plugin in this location
- restart koreader

### Pairing 
- enable wifi on the ereader
- go to the most left icon in the menu bar
- goto Syncthing
- set GUI Password to a value of choice
- enable syncthing (checkmark set)
- press syncthing Web Gui
- open browser on a device in the same wifi 
- enter the the ip shown on the ereader followed by a “:” and the port stated e.g. http://10.0.0.5:7536
- click on show id (screenshot1)
- copy the ip
- enter the user syncthing and the password chosen above
- open a second tab with the ip of the server and the port chosen for syncthing there e.g. http://10.0.0.10:8384
- klick add remote device
- paste the id from the ereader and save
- change back and accept the server
- click add folder and put the path to where the Books are stored on the ereader screenshot2 click on sharing and select the server to sync the folder with screenshot3 and save
- click add folder and put the path to koreaderon the ereader screenshot2 click on sharing and select the server to sync the folder with screenshot3 and save
- accept the share of the book folder and enter the path choosen above screenshot 4
- accept the share of the book koreader folder and enter the path choosen above screenshot 5

Now if wifi is on and syncthing enabled on the ereader it will sync the folders with the server so koshelf can use the data 

## KoShelf

runing the image with the following docker-compose.yml please use the path used above (with the added setting for the koreader path and env variables to the needs
```
---
services:
  koshelf:
    image: ghcr.io/devtigro/koshelf:latest
    container_name: koshelf
    volumes:
      - /opt/syncthing/koreader/settings:/settings:ro
      - /opt/syncthing/kobobooks:/books:ro
    ports:
      - 3000:3000
    environment:
      KOSHELF_TITLE: “My KoShelf”
      KOSHELF_TIMEZONE: Europe/Oslo
      KOSHELF_MIN_PAGES_PER_DAY: 3
      TZ: Europe/Oslo
    restart: “unless-stopped”
```

then open the browser on the server e.g. http://10.0.0.10:3000 to use KoShelf
