# ===========================================
#      Hungarian (hu) — Base Language File
# ===========================================
# This is the base translation file for Hungarian, using hu_HU.
# Regional variants should only override keys that differ.

# Machine-readable metadata (used by --list-languages)
-lang-code = hu
-lang-name = Magyar (Magyarország)
-lang-dialect = hu_HU

# -----------------------------------
#           Navigation & Shared
# -----------------------------------
books = Könyvek
comics = Képregények
statistics = Statisztikák
calendar = Naptár
recap = Visszatekintés
settings = Beállítások
github = GitHub
reading-companion = Olvasónapló
# Used in footer/sidebar for update time
last-updated = Utoljára frissítve:
view-details = Részletek
# -----------------------------------
#             Settings
# -----------------------------------
appearance-setting = Megjelenés
theme-setting = Téma
    .description = Válaszd ki a KoShelf megjelenését. Az "Automatikus" a rendszered beállítását követi.
    .option-auto = Automatikus
    .option-light = Világos
    .option-dark = Sötét
prefetch-setting = Linkek előtöltése
    .description = Oldalak előtöltése, amikor a linkekre viszed az egeret vagy ráböksz, a gyorsabb navigáció érdekében.
    .option-enabled = Engedélyezve
    .option-disabled = Letiltva
    .connection-note = Megjegyzés: Az előtöltés automatikusan kikapcsol, ha a kapcsolatod korlátozott (pl. adatforgalom-csökkentő mód vagy lassú mobilhálózat).
language-setting = Nyelv
    .hint = A felület fordításának nyelvét állítja be.
region-setting = Régió
    .hint = Befolyásolja a dátumok és számok formázását.
    .preview-date = Dátum előnézet
    .preview-number = Szám előnézet
    .majority-group = Leggyakoribb régiók
    .all-group = Minden támogatott régió

# -----------------------------------
#         Authentication
# -----------------------------------
login = Bejelentkezés
    .title = Jelentkezz be: { $site }
    .password = Jelszó
    .submit = Bejelentkezés
    .error = Érvénytelen jelszó
    .rate-limited = Túl sok próbálkozás. Kérjük, próbáld újra hamarosan.
change-password = Jelszó módosítása
    .setting = Jelszó
    .current = Jelenlegi jelszó
    .new = Új jelszó
    .confirm = Jelszó megerősítése
    .changed = A jelszó sikeresen módosítva
    .too-short = A jelszónak legalább { $min } karakter hosszúnak kell lennie
    .mismatch = A jelszavak nem egyeznek
    .incorrect = Hibás jelenlegi jelszó
    .current-placeholder = Jelenlegi jelszó megadása
    .new-placeholder = Min. 8 karakter
    .new-hint = Legalább 8 karakter hosszú legyen
    .confirm-placeholder = Új jelszó újra
session-management =
    .setting = Munkamenetek
    .current = Jelenlegi munkamenet
    .this-device = Ez az eszköz
    .last-active = Utoljára aktív
    .revoke = Visszavonás
    .revoke-confirm = Visszavonod ezt a munkamenetet?
    .revoked = Munkamenet visszavonva
    .device-info = { $browser } – { $os }
    .logout = Kijelentkezés

# -----------------------------------
#        Book List & Library
# -----------------------------------
search-placeholder = Keresés: könyv, szerző, sorozat...
filter =
    .aria-label = Könyvek szűrése
    .all = Összes
    .all-aria = { filter.aria-label } - Jelenleg: { filter.all }
    .reading = { status.reading }
    .reading-aria = { filter.aria-label } - Jelenleg: { filter.reading }
    .completed = { status.completed }
    .completed-aria = { filter.aria-label } - Jelenleg: { filter.completed }
    .unread = { status.unread }
    .unread-aria = { filter.aria-label } - Jelenleg: { filter.unread }
    .on-hold = { status.on-hold }
    .on-hold-aria = { filter.aria-label } - Jelenleg: { filter.on-hold }
no-books-found = Nem találhatók könyvek
no-books-match = Egyetlen könyv sem felel meg a keresési vagy szűrési feltételeknek.
try-adjusting = Próbáld meg módosítani a keresési vagy szűrési feltételeket
status =
    .reading = Most olvasom
    .reading-short = Olvasás
    .on-hold = Félretéve
    .completed = Befejezve
    .completed-short = Befejezett
    .unread = Olvasatlan
    .abandoned = Félbehagyott
book-label = { $count ->
   *[other] Könyv
}
comic-label = { $count ->
   *[other] Képregény
}
books-finished = { $count ->
   *[other] { book-label } befejezve
}
comics-finished = { $count ->
   *[other] { comic-label } befejezve
}
unknown-book = Ismeretlen könyv
unknown-author = Ismeretlen szerző
by = Írta:
book-overview = Könyv áttekintése
comic-overview = Képregény áttekintése

# -----------------------------------
#            Book Details
# -----------------------------------
description = Leírás
publisher = Kiadó
series = Sorozat
genres = Műfajok
language = Nyelv
book-identifiers = Könyv azonosítók
my-review = Értékelésem
my-note = Jegyzetem
highlights = Kiemelések
highlights-label = { $count ->
   *[other] Kiemelés
}
notes-label = { $count ->
   *[other] Jegyzet
}
bookmarks = Könyvjelzők
page-bookmark = Oldal könyvjelző
bookmark-anchor = Könyvjelző horgony
highlights-quotes = Kiemelések és idézetek
additional-information = További információk
reading-progress = Olvasási folyamat
page-number = { $count }. oldal
last-read = Utoljára olvasva
pages = { $count ->
   *[other] { $count } oldal
}
pages-label = { $count ->
   *[other] Oldal
}

# -----------------------------------
#       Statistics & Progress
# -----------------------------------
reading-statistics = Olvasási statisztikák
overall-statistics = Összesített statisztikák
weekly-statistics = Heti statisztikák
yearly-statistics = Éves statisztikák
total-read-time = Teljes olvasási idő
total-pages-read = Elolvasott oldalak száma
pages-per-hour = Oldal/óra
# Abbreviation for Pages Per Hour
pph-abbreviation = old./ó.
reading-sessions-label = { $count ->
   *[other] Olvasási alkalom
}
session =
    .longest = Leghosszabb olvasás
    .average = Átlagos olvasás
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /átl. olvasás
streak =
    .current = Jelenlegi sorozat
    .longest = Leghosszabb sorozat
reading-streak = Olvasási sorozat
average-time-day = Átlag idő/nap
average-pages-day = Átlag oldal/nap
most-pages-in-day = Legtöbb oldal egy nap
longest-daily-reading = Leghosszabb napi olvasás
reading-completions = Befejezett olvasások
completed-books = Befejezett könyvek
statistics-from-koreader = Statisztikák a KoReader munkameneteiből
reading-time = Olvasási idő
pages-read = Elolvasott oldalak
units-days = { $count ->
   *[other] { $count } nap
}
units-sessions = { $count ->
   *[other] { $count } olvasás
}

# -----------------------------------
#               Recap
# -----------------------------------
my-reading-recap = Olvasási visszatekintésem
share = Megosztás
    .recap-label = Összegző kép megosztása
download = Letöltés
    .recap-label = Összegző kép letöltése
recap-story = Történet (Story)
    .details = 1260 x 2240 — Függőleges 9:16
recap-square = Négyzet
    .details = 1500 x 1500 — Négyzet 1:1
recap-banner = Transzparens (Banner)
    .details = 2400 x 1260 — Vízszintes 2:1
best-month = Legjobb hónap
active-days = { $count ->
   *[other] Aktív nap
}
active-days-tooltip = { $count ->
   *[other] aktív nap
}
toggle =
    .hide = Elrejtés
    .show = Mutatás
less = Kevesebb
more = Több
period = Időszak
sessions = Olvasási alkalom
yearly-summary = Éves összegzés: { $count }
recap-empty =
    .nothing-here = Itt még nincs semmi
    .try-switching = Próbálj meg fentebb hatókört vagy évet váltani.
    .finish-reading = Fejezz be egy könyvet a KoReaderben, hogy lásd az összegzést.
    .info-question = Miért nem jelenik meg az összegzésem?
    .info-answer = A KoShelf az olvasási statisztikákat használja a befejezett könyvek és képregények érzékeléséhez, ami lehetővé teszi az újraolvasások nyomon követését. Ha egyszerűen "befejezett"-re állítasz egy könyvet olvasási adatok nélkül, az nem fog itt megjelenni.
stats-empty =
    .nothing-here = Itt még nincs semmi
    .start-reading = Kezdj el olvasni a KoReaderben, hogy itt lásd a statisztikáidat.
    .info-question = Hogyan működik az olvasás nyomon követése?
    .info-answer = A KoReader automatikusan rögzíti az olvasási munkameneteidet, beleértve az eltöltött időt és az elolvasott oldalakat. Szinkronizáld a statisztikai adatbázisodat a KoShelffel, hogy itt vizuálisan is lásd a tevékenységedet.
error-state =
    .title = Valami hiba történt
    .description =
        Az adatokat nem sikerült betölteni.
        Kérjük, próbáld újra.
    .not-found-title = Nem található
    .not-found-description = A keresett oldal nem létezik, vagy eltávolították.
    .connection-title = Csatlakozási hiba
    .connection-description =
        Nem sikerült elérni a szervert.
        Ellenőrizd az internetkapcsolatot, és próbáld újra.
    .file-unavailable-title = A könyvfájl nem érhető el
    .file-unavailable-description = A könyv adatai megvannak, de a könyvfájl hiányzik.
    .retry = Újrapróbálkozás
    .go-to-list = Ugrás ide: {$collection}
    .crash-title = Valami hiba történt
    .crash-description = Váratlan hiba történt az oldal megjelenítése közben.
    .crash-report = Hiba jelentése

# Navigation and sorting
sort-order =
    .aria-label-toggle = Rendezési sorrend megváltoztatása
    .newest-first = { sort-order.aria-label-toggle } - Jelenleg: Legújabb elöl
    .oldest-first = { sort-order.aria-label-toggle } - Jelenleg: Legrégebbi elöl
previous-month =
    .aria-label = Előző hónap
next-month =
    .aria-label = Következő hónap
search =
    .aria-label = Keresés
close-search =
    .aria-label = Keresés bezárása
go-back =
    .aria-label = Vissza

# -----------------------------------
#              Reader
# -----------------------------------
reader =
    .title = Olvasó
    .loading = Könyv betöltése…
    .previous-page = Előző oldal
    .next-page = Következő oldal
    .contents = Tartalom
open-at-annotation = Megnyitás a jelölésnél
open-in-reader = Megnyitás
    .aria-label = Megnyitás az olvasóban
reader-settings =
    .aria-label = Megjelenítési beállítások
    .typography = Tipográfia
    .margins = Margók
reader-font-size = Betűméret
    .decrease-aria = Betűméret csökkentése
    .increase-aria = Betűméret növelése
reader-line-spacing = Sortávolság
    .decrease-aria = Sortávolság csökkentése
    .increase-aria = Sortávolság növelése
reader-word-spacing = Szóköz
    .decrease-aria = Szóköz csökkentése
    .increase-aria = Szóköz növelése
reader-hyphenation = Elválasztás
reader-floating-punctuation = Lebegő írásjelek
reader-embedded-fonts = Beágyazott betűtípusok
reader-left-margin = Bal margó
    .decrease-aria = Bal margó csökkentése
    .increase-aria = Bal margó növelése
reader-right-margin = Jobb margó
    .decrease-aria = Jobb margó csökkentése
    .increase-aria = Jobb margó növelése
reader-top-margin = Felső margó
    .decrease-aria = Felső margó csökkentése
    .increase-aria = Felső margó növelése
reader-bottom-margin = Alsó margó
    .decrease-aria = Alsó margó csökkentése
    .increase-aria = Alsó margó növelése
reader-mode =
    .auto = Könyv
    .on = Be
    .off = Ki
reader-reset =
    .book = Könyvbeállítások használata
    .book-aria = Visszaállítás a könyv szinkronizált megjelenítési beállításaira
    .defaults = Visszaállítás alapértékekre
    .defaults-aria = Olvasó megjelenítési beállításainak visszaállítása alapértékekre
reader-drawer =
    .aria-label = Könyvnavigációs panel
reader-no-toc = Nem érhető el tartalomjegyzék
    .description = Ez a fájl nem tartalmaz fejezetjelölőket.
reader-no-highlights = Még nincsenek kiemelések
    .description = A KoReaderben hozzáadott kiemelések itt fognak megjelenni.
reader-no-bookmarks = Még nincsenek könyvjelzők
    .description = A KoReaderben hozzáadott könyvjelzők itt fognak megjelenni.
select-month = Hónap kiválasztása

# -----------------------------------
#           Time & Dates
# -----------------------------------
january = Január
    .short = Jan.
february = Február
    .short = Febr.
march = Március
    .short = Márc.
april = Április
    .short = Ápr.
may = Május
    .short = Máj.
june = Június
    .short = Jún.
july = Július
    .short = Júl.
august = Augusztus
    .short = Aug.
september = Szeptember
    .short = Szept.
october = Október
    .short = Okt.
november = November
    .short = Nov.
december = December
    .short = Dec.

# Weekday abbreviations (only Mon/Thu/Sun for GitHub-style heatmap visualization)
weekday =
    .mon = H
    .tue = K
    .wed = Sze
    .thu = Cs
    .fri = P
    .sat = Szo
    .sun = V

# Time units: w=weeks, d=days, h=hours, m=minutes
units =
    .w = h
    .d = n
    .h = ó
    .m = p
    .s = s
today = Ma
of-the-year = az évből
of = /
last = Utolsó

# Time unit labels (standalone word forms for displaying after numbers)
days_label = { $count ->
   *[other] nap
}
hours_label = { $count ->
   *[other] óra
}
minutes_label = { $count ->
   *[other] perc
}

# -----------------------------------
#         Writeback / Editing
# -----------------------------------
edit-warning =
    .title = Mielőtt szerkesztenél
    .body = Győződj meg róla, hogy a könyv be van zárva a KoReaderben, és elég idő eltelt a bezárás óta, hogy az esetleges fájlszinkronizálás (pl. Syncthing) befejeződjön. A KoReader felülírja a metaadat-fájlt, amíg a könyv nyitva van. Ha itt mentesz módosításokat, amíg a könyv még nyitva van — vagy mielőtt a legfrissebb sidecar fájl szinkronizálódott volna — a módosításaid elvesznek.
    .dismiss = Ne jelenjen meg többé ez a figyelmeztetés
    .understood = Megértettem
edit =
    .aria-label = Szerkesztés
save = Mentés
cancel = Mégse
delete = Törlés
no-review-available = Nincs elérhető értékelés
    .hint-edit = Használd a szerkesztés gombot az értékelésed hozzáadásához.
    .hint-readonly = Az értékelések közvetlenül a KoReaderben adhatók hozzá.
add-review = Értékelés hozzáadása
delete-review = Értékelés törlése
add-note = Jegyzet hozzáadása
edit-note = Jegyzet szerkesztése
delete-note = Jegyzet törlése
delete-highlight = Kiemelés törlése
delete-highlight-and-note = Kiemelés és jegyzet törlése
delete-bookmark = Könyvjelző törlése
change-status = Állapot módosítása
highlight-color =
    .aria-label = Kiemelés színe
highlight-drawer =
    .aria-label = Kiemelés stílusa
    .lighten = Kiemelés
    .underscore = Aláhúzás
    .strikeout = Áthúzás
    .invert = Invertálás

# -----------------------------------
#         Toast notifications
# -----------------------------------
toast-dismiss-label = Elvetés
toast-update-item-error = A módosítások mentése sikertelen. A szerkesztések visszaállítva.
toast-update-annotation-error = A jegyzet frissítése sikertelen. A módosítások visszaállítva.
toast-delete-annotation-error = A jegyzet törlése sikertelen. Visszaállítva.

# -----------------------------------
#            Page Activity
# -----------------------------------
page-activity = Oldalaktivitás
    .reading-number = { $number }. olvasás
    .select-reading = Olvasás kiválasztása
    .page = { $page }. oldal
    .unread = olvasatlan
    .visits = { $count ->
        [one] { $count } látogatás
       *[other] { $count } látogatás
    }
    .highlights = { $count ->
        [one] { $count } kiemelés
       *[other] { $count } kiemelés
    }
    .bookmarks = { $count ->
        [one] { $count } könyvjelző
       *[other] { $count } könyvjelző
    }
    .notes = { $count ->
        [one] { $count } jegyzet
       *[other] { $count } jegyzet
    }
    .of = { $total } oldalból
    .pages-read = oldal elolvasva
    .no-data = Ehhez az elemhez nem állnak rendelkezésre oldalszintű olvasási adatok.
    .legend-bookmark = Könyvjelző
    .legend-chapter = Fejezet
    .info = Az oldalszámokat a dokumentum aktuális formázási beállításai befolyásolják. A KoShelf többi részétől eltérően ez a nézet nem támogatja a szintetikus oldalszámozást.
