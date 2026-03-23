# ===========================================
#      Spanish (es) — Base Language File
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
settings = Configuración
github = GitHub
reading-companion = Compañero de lectura
# Used in footer/sidebar for update time
last-updated = Última actualización
view-details = Ver detalles

# -----------------------------------
#             Settings
# -----------------------------------
appearance-setting = Apariencia
theme-setting = Tema
    .description = Elige cómo se ve KoShelf. Automático sigue la preferencia de tu sistema.
    .option-auto = Automático
    .option-light = Claro
    .option-dark = Oscuro
prefetch-setting = Precarga de enlaces
    .description = Precarga páginas cuando pasas el cursor, enfocas o tocas enlaces para que la navegación se sienta más rápida.
    .option-enabled = Activado
    .option-disabled = Desactivado
    .connection-note = Nota: la precarga se omite automáticamente cuando tu conexión es limitada (por ejemplo, ahorro de datos o redes móviles lentas).
language-setting = Idioma
    .hint = Establece el idioma de todas las traducciones de la interfaz.
region-setting = Región
    .hint = Afecta el formato de fechas y números.
    .preview-date = Vista previa Fecha
    .preview-number = Vista previa Número
    .majority-group = Regiones de mayoría lingüística
    .all-group = Todas las regiones compatibles

# -----------------------------------
#         Authentication
# -----------------------------------
login = Iniciar sesión
    .title = Inicia sesión en { $site }
    .password = Contraseña
    .submit = Entrar
    .error = Contraseña incorrecta
    .rate-limited = Demasiados intentos. Inténtalo de nuevo en breve.
change-password = Cambiar contraseña
    .setting = Contraseña
    .current = Contraseña actual
    .new = Nueva contraseña
    .confirm = Confirmar contraseña
    .changed = Contraseña cambiada correctamente
    .too-short = La contraseña debe tener al menos { $min } caracteres
    .mismatch = Las contraseñas no coinciden
    .incorrect = La contraseña actual es incorrecta
    .current-placeholder = Ingresa la contraseña actual
    .new-placeholder = Mín. 8 caracteres
    .new-hint = Debe tener al menos 8 caracteres
    .confirm-placeholder = Reingresa la nueva contraseña
session-management =
    .setting = Sesiones
    .current = Sesión actual
    .this-device = Este dispositivo
    .last-active = Última actividad
    .revoke = Revocar
    .revoke-confirm = ¿Revocar esta sesión?
    .revoked = Sesión revocada
    .device-info = { $browser } en { $os }
    .logout = Cerrar sesión

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
    .reading-short = Leyendo
    .on-hold = En espera
    .completed = Completado
    .completed-short = Completado
    .unread = Sin leer
    .abandoned = Abandonado
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
notes-label = { $count ->
    [one] Nota
   *[other] Notas
}
bookmarks = Marcadores
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
reading-statistics = Estadísticas de lectura
overall-statistics = Estadísticas generales
weekly-statistics = Estadísticas semanales
yearly-statistics = Estadísticas anuales
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
average-time-day = Media tiempo/día
average-pages-day = Media páginas/día
most-pages-in-day = Más páginas en un día
longest-daily-reading = Lectura diaria más larga
reading-completions = Lecturas completadas
completed-books = Libros completados
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
active-days-tooltip = { $count ->
    [one] día activo
   *[other] días activos
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
error-state =
    .title = Algo salió mal
    .description =
        No se pudieron cargar los datos.
        Por favor, inténtalo de nuevo.
    .not-found-title = No encontrado
    .not-found-description = La página que buscas no existe o ha sido eliminada.
    .connection-title = Conexión fallida
    .connection-description =
        No se pudo contactar con el servidor.
        Comprueba tu conexión e inténtalo de nuevo.
    .file-unavailable-title = Archivo del libro no disponible
    .file-unavailable-description = Se encontraron los detalles del libro, pero falta el archivo del libro.
    .retry = Reintentar

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
go-back =
    .aria-label = Volver

# -----------------------------------
#              Reader
# -----------------------------------
reader =
    .title = Lector
    .loading = Cargando libro…
    .previous-page = Página anterior
    .next-page = Página siguiente
    .contents = Contenido
open-at-annotation = Abrir en la anotación
open-in-reader = Abrir
    .aria-label = Abrir en el lector
reader-settings =
    .aria-label = Ajustes de visualización
    .typography = Tipografía
    .margins = Márgenes
reader-font-size = Tamaño de fuente
    .decrease-aria = Disminuir tamaño de fuente
    .increase-aria = Aumentar tamaño de fuente
reader-line-spacing = Interlineado
    .decrease-aria = Disminuir interlineado
    .increase-aria = Aumentar interlineado
reader-word-spacing = Espaciado de palabras
    .decrease-aria = Disminuir espaciado de palabras
    .increase-aria = Aumentar espaciado de palabras
reader-hyphenation = Guionado
reader-floating-punctuation = Puntuación flotante
reader-embedded-fonts = Fuentes incrustadas
reader-left-margin = Margen izquierdo
    .decrease-aria = Disminuir margen izquierdo
    .increase-aria = Aumentar margen izquierdo
reader-right-margin = Margen derecho
    .decrease-aria = Disminuir margen derecho
    .increase-aria = Aumentar margen derecho
reader-top-margin = Margen superior
    .decrease-aria = Disminuir margen superior
    .increase-aria = Aumentar margen superior
reader-bottom-margin = Margen inferior
    .decrease-aria = Disminuir margen inferior
    .increase-aria = Aumentar margen inferior
reader-mode =
    .auto = Libro
    .on = Activado
    .off = Desactivado
reader-reset =
    .book = Usar ajustes del libro
    .book-aria = Restablecer a los ajustes de visualización sincronizados del libro
    .defaults = Restablecer valores predeterminados
    .defaults-aria = Restablecer la configuración de visualización del lector a los valores predeterminados
reader-drawer =
    .aria-label = Panel de navegación del libro
reader-no-toc = No hay tabla de contenido disponible
    .description = Este archivo no incluye marcadores de capítulo.
reader-no-highlights = Aún no hay subrayados
    .description = Los subrayados que agregues en KoReader aparecerán aquí.
reader-no-bookmarks = Aún no hay marcadores
    .description = Los marcadores que agregues en KoReader aparecerán aquí.
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
    .tue = Mar
    .wed = Mié
    .thu = Jue
    .fri = Vie
    .sat = Sáb
    .sun = Dom

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

# -----------------------------------
#         Writeback / Editing
# -----------------------------------
edit-warning =
    .title = Antes de editar
    .body = Asegúrate de que el libro esté cerrado en KoReader y haya estado cerrado el tiempo suficiente para que cualquier sincronización de archivos (por ejemplo, Syncthing) se complete. KoReader sobrescribe el archivo de metadatos cuando un libro está abierto. Si guardas cambios aquí mientras el libro sigue abierto — o antes de que el último archivo sidecar se haya sincronizado — tus cambios se perderán.
    .dismiss = No mostrar esta advertencia de nuevo
    .understood = Entendido
edit =
    .aria-label = Editar
save = Guardar
cancel = Cancelar
delete = Eliminar
no-review-available = Sin reseña disponible
    .hint-edit = Usa el botón de edición para agregar tu reseña y calificación.
    .hint-readonly = Las reseñas se pueden agregar directamente en KoReader.
add-review = Agregar reseña
delete-review = Eliminar reseña
add-note = Agregar nota
edit-note = Editar nota
delete-note = Eliminar nota
delete-highlight = Eliminar resaltado
delete-highlight-and-note = Eliminar resaltado y nota
delete-bookmark = Eliminar marcador
change-status = Cambiar estado
highlight-color =
    .aria-label = Color de resaltado
highlight-drawer =
    .aria-label = Estilo de resaltado
    .lighten = Resaltado
    .underscore = Subrayado
    .strikeout = Tachado
    .invert = Invertido

# -----------------------------------
#         Toast notifications
# -----------------------------------
toast-dismiss-label = Cerrar
toast-update-item-error = No se pudieron guardar los cambios. Tus ediciones han sido revertidas.
toast-update-annotation-error = No se pudo actualizar la anotación. Tus cambios han sido revertidos.
toast-delete-annotation-error = No se pudo eliminar la anotación. Ha sido restaurada.
