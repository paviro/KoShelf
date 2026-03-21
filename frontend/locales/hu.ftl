# ===========================================
#      Hungarian (hu) — Base Language File
# ===========================================
# This is the base translation file for Hungarian, using hu_HU.
# Regional variants should only override keys that differ.

# Machine-readable metadata (used by --list-languages)
-lang-code = hu
-lang-name = Magyar
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
appearance-setting = Megjelenés
theme-setting = Téma
theme-setting-description = Válaszd ki a KoShelf megjelenését. Az "Automatikus" a rendszered beállítását követi.
theme-option-auto = Automatikus
theme-option-light = Világos
theme-option-dark = Sötét
prefetch-setting = Linkek előtöltése
prefetch-setting-description = Oldalak előtöltése, amikor a linkekre viszed az egeret vagy ráböksz, a gyorsabb navigáció érdekében.
prefetch-option-enabled = Engedélyezve
prefetch-option-disabled = Letiltva
prefetch-setting-connection-note = Megjegyzés: Az előtöltés automatikusan kikapcsol, ha a kapcsolatod korlátozott (pl. adatforgalom-csökkentő mód vagy lassú mobilhálózat).
language-setting = Nyelv
region-setting = Régió
language-setting-hint = A felület fordításának nyelvét állítja be.
region-setting-hint = Befolyásolja a dátumok és számok formázását.
preview-date = Dátum előnézet
preview-number = Szám előnézet
region-setting-majority-group = Leggyakoribb régiók
region-setting-all-group = Minden támogatott régió
login = Bejelentkezés
login-title = Jelentkezz be: { $site }
login-password = Jelszó
login-submit = Bejelentkezés
login-error = Érvénytelen jelszó
login-rate-limited = Túl sok próbálkozás. Kérjük, próbáld újra hamarosan.
password-setting = Jelszó
change-password = Jelszó módosítása
current-password = Jelenlegi jelszó
new-password = Új jelszó
confirm-password = Jelszó megerősítése
password-changed = A jelszó sikeresen módosítva
password-too-short = A jelszónak legalább { $min } karakter hosszúnak kell lennie
password-mismatch = A jelszavak nem egyeznek
incorrect-password = Hibás jelenlegi jelszó
sessions-setting = Munkamenetek
current-session = Jelenlegi munkamenet
this-device = Ez az eszköz
last-active = Utoljára aktív
revoke-session = Visszavonás
revoke-session-confirm = Visszavonod ezt a munkamenetet?
session-revoked = Munkamenet visszavonva
session-device-info = { $browser } – { $os }
logout = Kijelentkezés
current-password-placeholder = Jelenlegi jelszó megadása
new-password-placeholder = Min. 8 karakter
new-password-hint = Legalább 8 karakter hosszú legyen
confirm-password-placeholder = Új jelszó újra

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
    .on-hold = Félretéve
    .completed = Befejezve
    .unread = Olvasatlan
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
open-reader-aria = Megnyitás az olvasóban
reader-title = Olvasó
reader-loading = Könyv betöltése…
reader-previous-page = Előző oldal
reader-next-page = Következő oldal
open-at-annotation = Megnyitás a jelölésnél
reader-contents = Tartalom
reader-settings-aria = Megjelenítési beállítások
reader-section-typography = Tipográfia
reader-section-margins = Margók
reader-font-size = Betűméret
reader-font-size-decrease-aria = Betűméret csökkentése
reader-font-size-increase-aria = Betűméret növelése
reader-line-spacing = Sortávolság
reader-line-spacing-decrease-aria = Sortávolság csökkentése
reader-line-spacing-increase-aria = Sortávolság növelése
reader-word-spacing = Szóköz
reader-word-spacing-decrease-aria = Szóköz csökkentése
reader-word-spacing-increase-aria = Szóköz növelése
reader-hyphenation = Elválasztás
reader-floating-punctuation = Lebegő írásjelek
reader-embedded-fonts = Beágyazott betűtípusok
reader-left-margin = Bal margó
reader-left-margin-decrease-aria = Bal margó csökkentése
reader-left-margin-increase-aria = Bal margó növelése
reader-right-margin = Jobb margó
reader-right-margin-decrease-aria = Jobb margó csökkentése
reader-right-margin-increase-aria = Jobb margó növelése
reader-top-margin = Felső margó
reader-top-margin-decrease-aria = Felső margó csökkentése
reader-top-margin-increase-aria = Felső margó növelése
reader-bottom-margin = Alsó margó
reader-bottom-margin-decrease-aria = Alsó margó csökkentése
reader-bottom-margin-increase-aria = Alsó margó növelése
reader-mode-auto = Könyv
reader-mode-on = Be
reader-mode-off = Ki
reader-reset-book = Könyvbeállítások használata
reader-reset-book-aria = Visszaállítás a könyv szinkronizált megjelenítési beállításaira
reader-reset-defaults = Visszaállítás alapértékekre
reader-reset-defaults-aria = Olvasó megjelenítési beállításainak visszaállítása alapértékekre
reader-drawer-aria = Könyvnavigációs panel
reader-no-toc = Nem érhető el tartalomjegyzék
reader-no-toc-description = Ez a fájl nem tartalmaz fejezetjelölőket.
reader-no-highlights = Még nincsenek kiemelések
reader-no-highlights-description = A KoReaderben hozzáadott kiemelések itt fognak megjelenni.
reader-no-bookmarks = Még nincsenek könyvjelzők
reader-no-bookmarks-description = A KoReaderben hozzáadott könyvjelzők itt fognak megjelenni.
open-in-reader = Megnyitás
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
