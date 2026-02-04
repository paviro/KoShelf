# ===========================================
#      English (en) — Base Language File
# ===========================================
# This is the base translation file for Spanish, using Spain vocabulary (es_ES).
# Regional variants (e.g., es_MX.ftl) should only override keys that differ.
#
#
# NEW KEYS: Always add new translation keys to en.ftl first, then other bases.

# Machine-readable metadata (used by --list-languages)
-lang-code = es
-lang-name = Español (España)
-lang-dialect = es_ES

# -----------------------------------
#           Navigation & Shared
# -----------------------------------
books = Libros
comics = Cómics
statistics = Estadísticas
calendar = Calendario
recap = Resumen
github = GitHub
loading = Cargando...
reload = Recargar
new-version-available = Nueva versión disponible
tap-to-reload = Toca para recargar
reading-companion = Compañero de lectura
# Used in footer/sidebar for update time
last-updated = Última actualización
view-details = Ver detalles

# -----------------------------------
#        Book List & Library
# -----------------------------------
search-placeholder = Buscar libro, autor, serie...
filter =
    .aria-label = Filtrar libros
    .all = Todos
    .all-aria = { filter.aria-label } - Actual: { filter.all }
    .reading = { status.reading }
    .reading-aria = { filter.aria-label } - Actual: { filter.reading }
    .completed = { status.completed }
    .completed-aria = { filter.aria-label } - Actual: { filter.completed }
    .unread = { status.unread }
    .unread-aria = { filter.aria-label } - Actual: { filter.unread }
    .on-hold = { status.on-hold }
    .on-hold-aria = { filter.aria-label } - Actual: { filter.on-hold }
no-books-found = No se encontraron libros
no-books-match = Ningún libro coincide con tu búsqueda o filtros actuales.
try-adjusting = Prueba a ajustar tu búsqueda o los filtros
status =
    .reading = Leyendo
    .on-hold = En espera
    .completed = Completado
    .unread = Sin leer
book-label = { $count ->
    [one] Libro
   *[other] Libros
}
comic-label = { $count ->
    [one] Cómic
   *[other] Cómics
}
books-finished = { $count ->
    [one] { book-label } terminado
   *[other] { book-label } terminados
}
comics-finished = { $count ->
    [one] { comic-label } terminado
   *[other] { comic-label } terminados
}
unknown-book = Libro desconocido
unknown-author = Autor desconocido
by = por
book-overview = Resumen del libro
comic-overview = Resumen del cómic

# -----------------------------------
#            Book Details
# -----------------------------------
description = Descripción
publisher = Editorial
series = Serie
genres = Géneros
language = Idioma
book-identifiers = Identificadores
my-review = Mi reseña
my-note = Mi nota
highlights = Subrayados
highlights-label = { $count ->
    [one] Subrayado
   *[other] Subrayados
}
notes = Notas
notes-label = { $count ->
    [one] Nota
   *[other] Notas
}
bookmarks = Marcadores
page = Página
page-bookmark = Marcador de página
bookmark-anchor = Ancla de marcador
highlights-quotes = Subrayados y Citas
additional-information = Información adicional
reading-progress = Progreso de lectura
page-number = Página { $count }
last-read = Leído por última vez
pages = { $count ->
    [one] { $count } página
   *[other] { $count } páginas
}
pages-label = { $count ->
    [one] Página
   *[other] Páginas
}

# -----------------------------------
#       Statistics & Progress
# -----------------------------------
# -----------------------------------
#        Statistics & Progress
# -----------------------------------
reading-statistics = Estadísticas de lectura
overall-statistics = Estadísticas generales
weekly-statistics = Estadísticas semanales
total-read-time = Tiempo total de lectura
total-pages-read = Total de páginas leídas
pages-per-hour = Páginas/Hora
# Abbreviation for Pages Per Hour
pph-abbreviation = pph
reading-sessions-label = { $count ->
    [one] Sesión de lectura
   *[other] Sesiones de lectura
}
session =
    .longest = Sesión más larga
    .average = Sesión media
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /sesión media
streak =
    .current = Racha actual
    .longest = Racha más larga
reading-streak = Racha de lectura
days-read = Días leídos
weekly-reading-time = Tiempo de lectura semanal
weekly-pages-read = Páginas leídas semanales
average-time-day = Media tiempo/día
average-pages-day = Media páginas/día
most-pages-in-day = Más páginas en un día
longest-daily-reading = Lectura diaria más larga
reading-completions = Lecturas completadas
statistics-from-koreader = Estadísticas de las sesiones de KoReader
reading-time = Tiempo de lectura
pages-read = Páginas leídas
units-days = { $count ->
    [one] { $count } día
   *[other] { $count } días
}
units-sessions = { $count ->
    [one] { $count } sesión
   *[other] { $count } sesiones
}

# -----------------------------------
#               Recap
# -----------------------------------
my-reading-recap = Mi resumen de KoShelf
share = Compartir
    .recap-label = Compartir imagen de resumen
download = Descargar
    .recap-label = Descargar imagen de resumen
recap-story = Historia
    .details = 1260 x 2240 — Vertical 9:16
recap-square = Cuadrado
    .details = 1500 x 1500 — Cuadrado 1:1
recap-banner = Banner
    .details = 2400 x 1260 — Horizontal 2:1
best-month = Mejor mes
active-days = { $count ->
    [one] Día activo
   *[other] Días activos
}
toggle =
    .hide = Ocultar
    .show = Mostrar
less = Menos
more = Más
period = Periodo
sessions = Sesiones
yearly-summary = Resumen anual { $count }
recap-empty =
    .nothing-here = Aún no hay nada aquí
    .try-switching = Prueba a cambiar el alcance o el año arriba.
    .finish-reading = Termina de leer en KoReader para ver tu resumen.
    .info-question = ¿Por qué no aparece mi resumen?
    .info-answer = KoShelf usa estadísticas de lectura para detectar libros y cómics completados, lo que permite rastrear relecturas. Marcar un libro como "terminado" sin datos de lectura no hará que aparezca aquí.
stats-empty =
    .nothing-here = Aún no hay nada aquí
    .start-reading = Empieza a leer con KoReader para ver tus estadísticas aquí.
    .info-question = ¿Cómo funciona el seguimiento de lectura?
    .info-answer = KoReader rastrea automáticamente tus sesiones de lectura, incluido el tiempo empleado y las páginas leídas. Sincroniza tu base de datos de estadísticas con KoShelf para ver tu actividad visualizada aquí.

# Navigation and sorting
sort-order =
    .aria-label-toggle = Alternar orden
    .newest-first = { sort-order.aria-label-toggle } - Actual: Más recientes primero
    .oldest-first = { sort-order.aria-label-toggle } - Actual: Más antiguos primero
previous-month =
    .aria-label = Mes anterior
next-month =
    .aria-label = Mes siguiente
search =
    .aria-label = Buscar
close-search =
    .aria-label = Cerrar búsqueda
close = Cerrar
    .aria-label = Cerrar
go-back =
    .aria-label = Volver
select-month = Seleccionar mes

# -----------------------------------
#           Time & Dates
# -----------------------------------
january = Enero
    .short = Ene
february = Febrero
    .short = Feb
march = Marzo
    .short = Mar
april = Abril
    .short = Abr
may = Mayo
    .short = May
june = Junio
    .short = Jun
july = Julio
    .short = Jul
august = Agosto
    .short = Ago
september = Septiembre
    .short = Sep
october = Octubre
    .short = Oct
november = Noviembre
    .short = Nov
december = Diciembre
    .short = Dic

# Weekday abbreviations (only Mon/Thu/Sun for GitHub-style heatmap visualization)
weekday =
    .mon = Lun
    .thu = Jue
    .sun = Dom

# Chrono date/time format strings (use %B for full month, %b for short, etc.)
# Note: Adapted to Spanish conventions (Day before Month)
datetime =
    .full = %-d de %B de %Y a las %H:%M
    .short-current-year = %-d de %b
    .short-with-year = %-d de %b de %Y

# Time units: w=weeks, d=days, h=hours, m=minutes
units =
    .w = sem
    .d = d
    .h = h
    .m = min
today = Hoy
of-the-year = del año
of = de
last = Último

# Time unit labels (standalone word forms for displaying after numbers)
days_label = { $count ->
    [one] día
   *[other] días
}
hours_label = { $count ->
    [one] hora
   *[other] horas
}
minutes_label = { $count ->
    [one] minuto
   *[other] minutos
}
