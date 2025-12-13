# ===========================================
#      Russian (ru) — Base Language File
# ===========================================
# This is the base translation file for Russian, using Standard Russian (ru_RU).
# Regional variants (e.g., ru_UA.ftl) should only override keys that differ.
#
# LOCALE HIERARCHY:
#   1. Regional variant (e.g., ru_UA.ftl) — sparse, only differences
#   2. Base language (this file) — complete translations
#   3. English fallback (en.ftl) — ultimate fallback for all languages
#
# NEW KEYS: Always add new translation keys to en.ftl first, then other bases.

# Machine-readable metadata (used by --list-languages)
-lang-code = ru
-lang-name = Русский (Россия)
-lang-dialect = ru_RU

# -----------------------------------
#           Navigation & Shared
# -----------------------------------
books = Книги
comics = Комиксы
statistics = Статистика
calendar = Календарь
recap = Итоги
github = GitHub
loading = Загрузка...
reload = Обновить
new-version-available = Доступна новая версия
tap-to-reload = Нажмите для обновления
reading-companion = Помощник чтения
# Used in footer/sidebar for update time
last-updated = Последнее обновление
view-details = Подробнее

# -----------------------------------
#        Book List & Library
# -----------------------------------
search-placeholder = Поиск книги, автора, серии...
filter =
    .aria-label = Фильтр книг
    .all = Все
    .all-aria = { filter.aria-label } - Текущий: { filter.all }
    .reading = { status.reading }
    .reading-aria = { filter.aria-label } - Текущий: { filter.reading }
    .completed = { status.completed }
    .completed-aria = { filter.aria-label } - Текущий: { filter.completed }
    .unread = { status.unread }
    .unread-aria = { filter.aria-label } - Текущий: { filter.unread }
    .on-hold = { status.on-hold }
    .on-hold-aria = { filter.aria-label } - Текущий: { status.on-hold }
no-books-found = Книги не найдены
no-books-match = Нет книг, соответствующих вашему поиску или фильтру.
try-adjusting = Попробуйте изменить критерии поиска или фильтра
status =
    .reading = Читаю сейчас
    .on-hold = Отложено
    .completed = Прочитано
    .unread = Не прочитано
book-label = { $count ->
    [one] Книга
    [few] Книги
    [many] Книг
   *[other] Книг
}
comic-label = { $count ->
    [one] Комикс
    [few] Комикса
    [many] Комиксов
   *[other] Комиксов
}
books-finished = { $count ->
    [one] { book-label } прочитана
    [few] { book-label } прочитаны
    [many] { book-label } прочитано
   *[other] { book-label } прочитано
}
comics-finished = { $count ->
    [one] { comic-label } прочитан
    [few] { comic-label } прочитаны
    [many] { comic-label } прочитано
   *[other] { comic-label } прочитано
}
unknown-book = Неизвестная книга
unknown-author = Неизвестный автор
by = автор
book-overview = Обзор книги
comic-overview = Обзор комикса

# -----------------------------------
#            Book Details
# -----------------------------------
description = Описание
publisher = Издательство
series = Серия
genres = Жанры
language = Язык
book-identifiers = Идентификаторы книги
my-review = Мой отзыв
my-note = Моя заметка
highlights = Выделения
highlights-label = { $count ->
    [one] Выделение
    [few] Выделения
    [many] Выделений
   *[other] Выделений
}
notes = Заметки
notes-label = { $count ->
    [one] Заметка
    [few] Заметки
    [many] Заметок
   *[other] Заметок
}
bookmarks = Закладки
page = Страница
page-bookmark = Закладка страницы
bookmark-anchor = Якорь закладки
highlights-quotes = Выделения и цитаты
additional-information = Дополнительная информация
reading-progress = Прогресс чтения
page-number = Страница { $count }
last-read = Последнее чтение
pages = { $count ->
    [one] { $count } страница
    [few] { $count } страницы
    [many] { $count } страниц
   *[other] { $count } страниц
}
pages-label = { $count ->
    [one] Страница
    [few] Страницы
    [many] Страниц
   *[other] Страниц
}

# -----------------------------------
#       Statistics & Progress
# -----------------------------------
reading-statistics = Статистика чтения
overall-statistics = Общая статистика
weekly-statistics = Недельная статистика
total-read-time = Общее время чтения
total-pages-read = Всего прочитано страниц
pages-per-hour = Страниц/час
# Abbreviation for Pages Per Hour
pph-abbreviation = стр/ч
reading-sessions-label = { $count ->
    [one] Сессия чтения
    [few] Сессии чтения
    [many] Сессий чтения
   *[other] Сессий чтения
}
session =
    .longest = Самая длинная сессия
    .average = Средняя сессия
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /сред. сессия
streak =
    .current = Текущая серия
    .longest = Самая длинная серия
reading-streak = Серия чтения
days-read = Дней прочитано
weekly-reading-time = Недельное время чтения
weekly-pages-read = Страниц за неделю
average-time-day = Среднее время/день
average-pages-day = Средние страницы/день
most-pages-in-day = Больше всего страниц за день
longest-daily-reading = Самое длинное чтение за день
reading-completions = Завершённые чтения
statistics-from-koreader = Статистика из сессий чтения KoReader
reading-time = Время чтения
pages-read = Прочитано страниц
units-days = { $count ->
    [one] { $count } день
    [few] { $count } дня
    [many] { $count } дней
   *[other] { $count } дней
}
units-sessions = { $count ->
    [one] { $count } сессия
    [few] { $count } сессии
    [many] { $count } сессий
   *[other] { $count } сессий
}

# -----------------------------------
#               Recap
# -----------------------------------
my-reading-recap = Мои итоги чтения KoShelf
share = Поделиться
    .recap-label = Поделиться изображением итогов
download = Скачать
    .recap-label = Скачать изображение итогов
recap-story = История
    .details = 1260 x 2240 — Вертикальный 9:16
recap-square = Квадрат
    .details = 2160 x 2160 — Квадрат 1:1
recap-banner = Баннер
    .details = 2400 x 1260 — Горизонтальный 2:1
best-month = Лучший месяц
active-days = { $count ->
    [one] Активный день
    [few] Активных дня
    [many] Активных дней
   *[other] Активных дней
}
toggle =
    .hide = Скрыть
    .show = Показать
less = Меньше
more = Больше
period = Период
sessions = Сессии
yearly-summary = Итог { $count } года
recap-empty =
    .nothing-here = Пока ничего нет
    .try-switching = Попробуйте изменить область или год выше.
    .finish-reading = Завершите чтение в KoReader, чтобы увидеть ваши итоги.
    .info-question = Почему мои итоги не отображаются?
    .info-answer = KoShelf использует статистику чтения для определения завершений книг и комиксов, что позволяет отслеживать перечитывания. Простое пометка книги как "завершённой" без данных о чтении не приведёт к её появлению здесь.
stats-empty =
    .nothing-here = Пока ничего нет
    .start-reading = Начните читать в KoReader, чтобы увидеть вашу статистику здесь.
    .info-question = Как работает отслеживание чтения?
    .info-answer = KoReader автоматически отслеживает ваши сессии чтения, включая потраченное время и прочитанные страницы. Синхронизируйте базу данных статистики с KoShelf, чтобы увидеть вашу активность визуализированной здесь.

# Navigation and sorting
sort-order =
    .aria-label-toggle = Переключить порядок сортировки
    .newest-first = { sort-order.aria-label-toggle } - Текущий: Сначала новые
    .oldest-first = { sort-order.aria-label-toggle } - Текущий: Сначала старые
previous-month =
    .aria-label = Предыдущий месяц
next-month =
    .aria-label = Следующий месяц
search =
    .aria-label = Поиск
close-search =
    .aria-label = Закрыть поиск
close = Закрыть
    .aria-label = Закрыть
go-back =
    .aria-label = Назад

# -----------------------------------
#           Time & Dates
# -----------------------------------
january = Январь
    .short = Янв
february = Февраль
    .short = Фев
march = Март
    .short = Мар
april = Апрель
    .short = Апр
may = Май
    .short = Май
june = Июнь
    .short = Июн
july = Июль
    .short = Июл
august = Август
    .short = Авг
september = Сентябрь
    .short = Сен
october = Октябрь
    .short = Окт
november = Ноябрь
    .short = Ноя
december = Декабрь
    .short = Дек

# Weekday abbreviations (only Mon/Thu/Sun for GitHub-style heatmap visualization)
weekday =
    .mon = Пн
    .thu = Чт
    .sun = Вс

# Chrono date/time format strings (use %B for full month, %b for short, etc.)
datetime =
    .full = %-d %B %Y в %H:%M
    .short-current-year = %-d %b
    .short-with-year = %-d %b %Y

# Time units: h=hours, m=minutes, d=days
units =
    .h = ч
    .m = мин
    .d = д

today = Сегодня
of-the-year = года
of = из
last = Последний

# Time unit labels (standalone word forms for displaying after numbers)
days_label = { $count ->
    [one] день
    [few] дня
    [many] дней
   *[other] дней
}
hours_label = { $count ->
    [one] час
    [few] часа
    [many] часов
   *[other] часов
}
minutes_label = { $count ->
    [one] минута
    [few] минуты
    [many] минут
   *[other] минут
}

