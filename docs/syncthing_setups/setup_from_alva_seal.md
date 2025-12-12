# Koreader Syncthing Docker Setup

> [!NOTE]
> This is the syncthing setup of [alva](https://github.com/alva-seal). Thank you for sharing and documenting it!

Although there are many ways to use this tool, here is one of them:

**Syncthing Sync:** I use Syncthing to sync both my books folder and KoReader settings folder from my Kobo to my server.

**Books and Statistics:** I point to the synced books folder with `--books-path` and to `statistics.sqlite3` in the synced KoReader settings folder with `--statistics-db`.

**Use of Docker image:** [KoShelf Docker](https://github.com/DevTigro/koshelf-docker)


## Syncthing

### On the server/pc where KoShelf is running

**Running the [syncthing](https://docs.linuxserver.io/images/docker-syncthing/) docker image:**

Setting of the path for the volumes to the needs of the user

```yaml
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

### Syncthing for KOReader

This describes the steps on Kobo devices (path for other devices may need adaptation).

It is assumed that KOReader is already installed and running.

- Download the Syncthing [plugin](https://github.com/jasonchoimtt/koreader-syncthing/archive/refs/heads/main.zip)
- Attach the Kobo to a device via USB
- Go to the hidden folder `.adds` on the e-reader
- Go to the sub folder `koreader`
- Go to the sub folder `plugins`
- Put the downloaded zip file in the plugins folder
- Unzip the plugin in this location
- Restart KOReader

### Pairing

- Enable Wi-Fi on the e-reader
- Go to the leftmost icon in the menu bar
- Go to Syncthing
- Set GUI Password to a value of choice
- Enable Syncthing (checkmark set)
- Press Syncthing Web GUI
- Open browser on a device on the same Wi-Fi
- Enter the IP shown on the e-reader followed by a “:” and the port stated e.g. http://10.0.0.5:7536
- Click on Show ID ![IMG_0108](https://github.com/user-attachments/assets/1380e7ac-3300-472a-a657-e46569767c80)
- Copy the IP
- Enter the user `syncthing` and the password chosen above
- Open a second tab with the IP of the server and the port chosen for Syncthing there e.g. http://10.0.0.10:8384
- Click Add Remote Device
- Paste the ID from the e-reader and save
- Switch back and accept the server
- Click Add Folder and put the path to where the Books are stored on the e-reader ![IMG_0108](https://github.com/user-attachments/assets/7f12a134-ba83-4758-b3d2-277b119b032c) click on sharing and select the server to sync the folder with. ![IMG_0113](https://github.com/user-attachments/assets/bdf7671e-806c-4c1e-b7a8-04b88fd26b03) and save.
- Click Add Folder and put the path to KOReader on the e-reader ![IMG_0111](https://github.com/user-attachments/assets/0d1c2766-d5aa-469b-ad89-e947374b303f) click on sharing and select the server to sync the folder with. ![IMG_0113](https://github.com/user-attachments/assets/49de9708-3ec1-46ea-887a-2a8f5d997669) and save.
- Accept the share of the book folder ![IMG_0114](https://github.com/user-attachments/assets/e3e936a7-8c50-4d36-9a69-20be26dce59a) and enter the path chosen above ![IMG_0115](https://github.com/user-attachments/assets/32489749-1d04-4e62-8183-c8987233272f)
- Accept the share of the KOReader folder ![IMG_0114](https://github.com/user-attachments/assets/14f2b86b-3fe8-4c89-a7d5-95cbdc79849b) and enter the path chosen above ![IMG_0116](https://github.com/user-attachments/assets/3e591699-5dfa-47c2-b1fa-eded9d6d5cf2)

Now if Wi-Fi is on and Syncthing enabled on the e-reader it will sync the folders with the server so KoShelf can use the data. 

## KoShelf

Running the [koshelf-docker](https://github.com/DevTigro/koshelf-docker) image with the following `docker-compose.yml`. Please use the paths used above (adjusting the KOReader path and env variables to your needs):

```yaml
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
      KOSHELF_TITLE: "My KoShelf"
      KOSHELF_TIMEZONE: Europe/Oslo
      KOSHELF_MIN_PAGES_PER_DAY: 3
      TZ: Europe/Oslo
    restart: unless-stopped
```

Then open the browser on the server e.g. http://10.0.0.10:3000 to use KoShelf.
