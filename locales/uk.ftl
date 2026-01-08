# ===========================================
#      Ukrainian (uk) — Base Language File
# ===========================================
# This is the base translation file for Ukrainian, using Standard Ukrainian (uk_UA).
# Regional variants (e.g., uk_RU.ftl) should only override keys that differ.
#
# LOCALE HIERARCHY:
#   1. Regional variant (e.g., uk_RU.ftl) — sparse, only differences
#   2. Base language (this file) — complete translations
#   3. English fallback (en.ftl) — ultimate fallback for all languages
#
# NEW KEYS: Always add new translation keys to en.ftl first, then other bases.

# Machine-readable metadata (used by --list-languages)
-lang-code = uk
-lang-name = Українська (Україна)
-lang-dialect = uk_UA

# -----------------------------------
#           Navigation & Shared
# -----------------------------------
books = Книги
comics = Комікси
statistics = Статистика
calendar = Календар
recap = Підсумки
github = GitHub
loading = Завантаження...
reload = Оновити
new-version-available = Доступна нова версія
tap-to-reload = Натисніть для оновлення
reading-companion = Супутник читання
# Used in footer/sidebar for update time
last-updated = Останнє оновлення
view-details = Детальніше

# -----------------------------------
#        Book List & Library
# -----------------------------------
search-placeholder = Пошук книги, автора, серії...
filter =
    .aria-label = Фільтр книг
    .all = Всі
    .all-aria = { filter.aria-label } - Поточний: { filter.all }
    .reading = { status.reading }
    .reading-aria = { filter.aria-label } - Поточний: { filter.reading }
    .completed = { status.completed }
    .completed-aria = { filter.aria-label } - Поточний: { filter.completed }
    .unread = { status.unread }
    .unread-aria = { filter.aria-label } - Поточний: { filter.unread }
    .on-hold = { status.on-hold }
    .on-hold-aria = { filter.aria-label } - Поточний: { status.on-hold }
no-books-found = Книги не знайдено
no-books-match = Немає книг, що відповідають вашому пошуку або фільтру.
try-adjusting = Спробуйте змінити критерії пошуку або фільтру
status =
    .reading = Читаю зараз
    .on-hold = Відкладено
    .completed = Прочитано
    .unread = Не прочитано
book-label = { $count ->
    [one] Книга
    [few] Книги
    [many] Книг
   *[other] Книг
}
comic-label = { $count ->
    [one] Комікс
    [few] Комікси
    [many] Коміксів
   *[other] Коміксів
}
books-finished = { $count ->
    [one] { book-label } прочитана
    [few] { book-label } прочитані
    [many] { book-label } прочитано
   *[other] { book-label } прочитано
}
comics-finished = { $count ->
    [one] { comic-label } прочитаний
    [few] { comic-label } прочитані
    [many] { comic-label } прочитано
   *[other] { comic-label } прочитано
}
unknown-book = Невідома книга
unknown-author = Невідомий автор
by = автор
book-overview = Огляд книги
comic-overview = Огляд коміксу

# -----------------------------------
#            Book Details
# -----------------------------------
description = Опис
publisher = Видавництво
series = Серія
genres = Жанри
language = Мова
book-identifiers = Ідентифікатори книги
my-review = Мій відгук
my-note = Моя нотатка
highlights = Виділення
highlights-label = { $count ->
    [one] Виділення
    [few] Виділення
    [many] Виділень
   *[other] Виділень
}
notes = Нотатки
notes-label = { $count ->
    [one] Нотатка
    [few] Нотатки
    [many] Нотаток
   *[other] Нотаток
}
bookmarks = Закладки
page = Сторінка
page-bookmark = Закладка сторінки
bookmark-anchor = Якір закладки
highlights-quotes = Виділення та цитати
additional-information = Додаткова інформація
reading-progress = Прогрес читання
page-number = Сторінка { $count }
last-read = Останнє читання
pages = { $count ->
    [one] { $count } сторінка
    [few] { $count } сторінки
    [many] { $count } сторінок
   *[other] { $count } сторінок
}
pages-label = { $count ->
    [one] Сторінка
    [few] Сторінки
    [many] Сторінок
   *[other] Сторінок
}

# -----------------------------------
#       Statistics & Progress
# -----------------------------------
reading-statistics = Статистика читання
overall-statistics = Загальна статистика
weekly-statistics = Тижнева статистика
total-read-time = Загальний час читання
total-pages-read = Всього прочитано сторінок
pages-per-hour = Сторінок/година
# Abbreviation for Pages Per Hour
pph-abbreviation = стор/год
reading-sessions-label = { $count ->
    [one] Сесія читання
    [few] Сесії читання
    [many] Сесій читання
   *[other] Сесій читання
}
session =
    .longest = Найдовша сесія
    .average = Середня сесія
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /серед. сесія
streak =
    .current = Поточна серія
    .longest = Найдовша серія
reading-streak = Серія читання
days-read = Днів прочитано
weekly-reading-time = Тижневий час читання
weekly-pages-read = Сторінок за тиждень
average-time-day = Середній час/день
average-pages-day = Середні сторінки/день
most-pages-in-day = Найбільше сторінок за день
longest-daily-reading = Найдовше денне читання
reading-completions = Завершені читання
statistics-from-koreader = Статистика з сесій читання KoReader
reading-time = Час читання
pages-read = Прочитано сторінок
units-days = { $count ->
    [one] { $count } день
    [few] { $count } дні
    [many] { $count } днів
   *[other] { $count } днів
}
units-sessions = { $count ->
    [one] { $count } сесія
    [few] { $count } сесії
    [many] { $count } сесій
   *[other] { $count } сесій
}

# -----------------------------------
#               Recap
# -----------------------------------
my-reading-recap = Мої підсумки читання KoShelf
share = Поділитися
    .recap-label = Поділитися зображенням підсумків
download = Завантажити
    .recap-label = Завантажити зображення підсумків
recap-story = Історія
    .details = 1260 x 2240 — Вертикальний 9:16
recap-square = Квадрат
    .details = 1500 x 1500 — Квадрат 1:1
recap-banner = Банер
    .details = 2400 x 1260 — Горизонтальний 2:1
best-month = Найкращий місяць
active-days = { $count ->
    [one] Активний день
    [few] Активні дні
    [many] Активних днів
   *[other] Активних днів
}
toggle =
    .hide = Приховати
    .show = Показати
less = Менше
more = Більше
period = Період
sessions = Сесії
yearly-summary = Річний підсумок { $count }
recap-empty =
    .nothing-here = Поки нічого немає
    .try-switching = Спробуйте змінити область або рік вище.
    .finish-reading = Завершіть читання в KoReader, щоб побачити ваші підсумки.
    .info-question = Чому мої підсумки не відображаються?
    .info-answer = KoShelf використовує статистику читання для визначення завершень книг і коміксів, що дозволяє відстежувати перечитування. Просте позначення книги як "завершеної" без даних про читання не призведе до її появи тут.
stats-empty =
    .nothing-here = Поки нічого немає
    .start-reading = Почніть читати в KoReader, щоб побачити вашу статистику тут.
    .info-question = Як працює відстеження читання?
    .info-answer = KoReader автоматично відстежує ваші сесії читання, включаючи витрачений час та прочитані сторінки. Синхронізуйте базу даних статистики з KoShelf, щоб побачити вашу активність візуалізованою тут.

# Navigation and sorting
sort-order =
    .aria-label-toggle = Перемкнути порядок сортування
    .newest-first = { sort-order.aria-label-toggle } - Поточний: Спочатку нові
    .oldest-first = { sort-order.aria-label-toggle } - Поточний: Спочатку старі
previous-month =
    .aria-label = Попередній місяць
next-month =
    .aria-label = Наступний місяць
search =
    .aria-label = Пошук
close-search =
    .aria-label = Закрити пошук
close = Закрити
    .aria-label = Закрити
go-back =
    .aria-label = Назад
select-month = Вибрати місяць

# -----------------------------------
#           Time & Dates
# -----------------------------------
january = Січень
    .short = Січ
february = Лютий
    .short = Лют
march = Березень
    .short = Бер
april = Квітень
    .short = Кві
may = Травень
    .short = Тра
june = Червень
    .short = Чер
july = Липень
    .short = Лип
august = Серпень
    .short = Сер
september = Вересень
    .short = Вер
october = Жовтень
    .short = Жов
november = Листопад
    .short = Лис
december = Грудень
    .short = Гру

# Weekday abbreviations (only Mon/Thu/Sun for GitHub-style heatmap visualization)
weekday =
    .mon = Пн
    .thu = Чт
    .sun = Нд

# Chrono date/time format strings (use %B for full month, %b for short, etc.)
datetime =
    .full = %-d %B %Y о %H:%M
    .short-current-year = %-d %b
    .short-with-year = %-d %b %Y

# Time units: w=weeks, d=days, h=hours, m=minutes
units =
    .w = тиж
    .d = дн
    .h = год
    .m = хв
today = Сьогодні
of-the-year = року
of = з
last = Останній

# Time unit labels (standalone word forms for displaying after numbers)
days_label = { $count ->
    [one] день
    [few] дні
    [many] днів
   *[other] днів
}
hours_label = { $count ->
    [one] година
    [few] години
    [many] годин
   *[other] годин
}
minutes_label = { $count ->
    [one] хвилина
    [few] хвилини
    [many] хвилин
   *[other] хвилин
}

