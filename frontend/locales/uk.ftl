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
settings = Налаштування
github = GitHub
reading-companion = Супутник читання
# Used in footer/sidebar for update time
last-updated = Останнє оновлення
view-details = Детальніше

# -----------------------------------
#             Settings
# -----------------------------------
appearance-setting = Оформлення
theme-setting = Тема
    .description = Виберіть вигляд KoShelf. Автоматично враховує системну тему.
    .option-auto = Автоматично
    .option-light = Світла
    .option-dark = Темна
prefetch-setting = Попереднє завантаження посилань
    .description = Попередньо завантажуйте сторінки під час наведення, фокусу або торкання посилань, щоб навігація відчувалася швидшою.
    .option-enabled = Увімкнено
    .option-disabled = Вимкнено
    .connection-note = Примітка: попереднє завантаження все одно автоматично вимикається за обмеженого зʼєднання (наприклад, режим економії трафіку або повільніші мобільні мережі).
language-setting = Мова
    .hint = Визначає мову для всіх перекладів інтерфейсу.
region-setting = Регіон
    .hint = Впливає на формат дат і чисел.
    .preview-date = Попередній перегляд Дата
    .preview-number = Попередній перегляд Число
    .majority-group = Регіони з мовною більшістю
    .all-group = Усі підтримувані регіони

# -----------------------------------
#         Authentication
# -----------------------------------
login = Увійти
    .title = Увійдіть до { $site }
    .password = Пароль
    .submit = Увійти
    .error = Неправильний пароль
    .rate-limited = Забагато спроб. Спробуйте ще раз трохи згодом.
change-password = Змінити пароль
    .setting = Пароль
    .current = Поточний пароль
    .new = Новий пароль
    .confirm = Підтвердьте пароль
    .changed = Пароль успішно змінено
    .too-short = Пароль має містити щонайменше { $min } символів
    .mismatch = Паролі не збігаються
    .incorrect = Неправильний поточний пароль
    .current-placeholder = Введіть поточний пароль
    .new-placeholder = Мін. 8 символів
    .new-hint = Має містити щонайменше 8 символів
    .confirm-placeholder = Повторіть новий пароль
session-management =
    .setting = Сеанси
    .current = Поточний сеанс
    .this-device = Цей пристрій
    .last-active = Остання активність
    .revoke = Відкликати
    .revoke-confirm = Відкликати цей сеанс?
    .revoked = Сеанс відкликано
    .device-info = { $browser } на { $os }
    .logout = Вийти

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
    .on-hold-aria = { filter.aria-label } - Поточний: { filter.on-hold }
no-books-found = Книги не знайдено
no-books-match = Немає книг, що відповідають вашому пошуку або фільтру.
try-adjusting = Спробуйте змінити критерії пошуку або фільтру
status =
    .reading = Читаю зараз
    .reading-short = Читаю
    .on-hold = Відкладено
    .completed = Прочитано
    .completed-short = Завершено
    .unread = Не прочитано
    .abandoned = Покинуто
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
notes-label = { $count ->
    [one] Нотатка
    [few] Нотатки
    [many] Нотаток
   *[other] Нотаток
}
bookmarks = Закладки
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
yearly-statistics = Річна статистика
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
average-time-day = Середній час/день
average-pages-day = Середні сторінки/день
most-pages-in-day = Найбільше сторінок за день
longest-daily-reading = Найдовше денне читання
reading-completions = Завершені читання
completed-books = Завершені книги
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
active-days-tooltip = { $count ->
    [one] активний день
    [few] активні дні
    [many] активних днів
   *[other] активних днів
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
error-state =
    .title = Щось пішло не так
    .description =
        Не вдалося завантажити дані.
        Будь ласка, спробуйте ще раз.
    .not-found-title = Не знайдено
    .not-found-description = Сторінка, яку ви шукаєте, не існує або була видалена.
    .connection-title = Помилка з'єднання
    .connection-description =
        Не вдалося зв'язатися з сервером.
        Перевірте з'єднання та спробуйте знову.
    .file-unavailable-title = Файл книги недоступний
    .file-unavailable-description = Дані книги знайдено, але файл книги відсутній.
    .retry = Спробувати знову
    .go-to-list = Відкрити {$collection}
    .crash-title = Щось пішло не так
    .crash-description = Під час відображення цієї сторінки сталася неочікувана помилка.
    .crash-report = Повідомити про проблему

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
go-back =
    .aria-label = Назад

# -----------------------------------
#              Reader
# -----------------------------------
reader =
    .title = Читач
    .loading = Завантаження книги…
    .previous-page = Попередня сторінка
    .next-page = Наступна сторінка
    .contents = Зміст
open-at-annotation = Відкрити на анотації
open-in-reader = Відкрити
    .aria-label = Відкрити у читачі
reader-settings =
    .aria-label = Налаштування відображення
    .typography = Типографіка
    .margins = Відступи
reader-font-size = Розмір шрифту
    .decrease-aria = Зменшити розмір шрифту
    .increase-aria = Збільшити розмір шрифту
reader-line-spacing = Міжрядковий інтервал
    .decrease-aria = Зменшити міжрядковий інтервал
    .increase-aria = Збільшити міжрядковий інтервал
reader-word-spacing = Міжслівний інтервал
    .decrease-aria = Зменшити міжслівний інтервал
    .increase-aria = Збільшити міжслівний інтервал
reader-hyphenation = Переноси
reader-floating-punctuation = Висяча пунктуація
reader-embedded-fonts = Вбудовані шрифти
reader-left-margin = Лівий відступ
    .decrease-aria = Зменшити лівий відступ
    .increase-aria = Збільшити лівий відступ
reader-right-margin = Правий відступ
    .decrease-aria = Зменшити правий відступ
    .increase-aria = Збільшити правий відступ
reader-top-margin = Верхній відступ
    .decrease-aria = Зменшити верхній відступ
    .increase-aria = Збільшити верхній відступ
reader-bottom-margin = Нижній відступ
    .decrease-aria = Зменшити нижній відступ
    .increase-aria = Збільшити нижній відступ
reader-mode =
    .auto = Книга
    .on = Увімк.
    .off = Вимк.
reader-reset =
    .book = Налаштування книги
    .book-aria = Повернути синхронізовані налаштування відображення книги
    .defaults = Скинути до типових
    .defaults-aria = Скинути налаштування відображення рідера до типових
reader-drawer =
    .aria-label = Панель навігації книгою
reader-no-toc = Зміст недоступний
    .description = У цьому файлі немає позначок розділів.
reader-no-highlights = Ще немає виділень
    .description = Виділення, які ви додасте в KoReader, з'являться тут.
reader-no-bookmarks = Ще немає закладок
    .description = Закладки, які ви додасте в KoReader, з'являться тут.
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
    .tue = Вт
    .wed = Ср
    .thu = Чт
    .fri = Пт
    .sat = Сб
    .sun = Нд

# Time units: w=weeks, d=days, h=hours, m=minutes
units =
    .w = тиж
    .d = дн
    .h = год
    .m = хв
    .s = s
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

# -----------------------------------
#         Writeback / Editing
# -----------------------------------
edit-warning =
    .title = Перед редагуванням
    .body = Переконайтеся, що книга закрита в KoReader і була закрита достатньо довго, щоб синхронізація файлів (наприклад, Syncthing) завершилася. KoReader перезаписує файл метаданих, коли книга відкрита. Якщо ви збережете зміни тут, поки книга ще відкрита — або до того, як останній sidecar-файл синхронізувався — ваші зміни будуть втрачені.
    .dismiss = Більше не показувати це попередження
    .understood = Зрозуміло
edit =
    .aria-label = Редагувати
save = Зберегти
cancel = Скасувати
delete = Видалити
no-review-available = Рецензія відсутня
    .hint-edit = Використовуйте кнопку редагування, щоб додати рецензію та оцінку.
    .hint-readonly = Рецензії можна додавати безпосередньо в KoReader.
add-review = Додати рецензію
delete-review = Видалити рецензію
add-note = Додати нотатку
edit-note = Редагувати нотатку
delete-note = Видалити нотатку
delete-highlight = Видалити виділення
delete-highlight-and-note = Видалити виділення та нотатку
delete-bookmark = Видалити закладку
change-status = Змінити статус
highlight-color =
    .aria-label = Колір виділення
highlight-drawer =
    .aria-label = Стиль виділення
    .lighten = Виділення
    .underscore = Підкреслення
    .strikeout = Закреслення
    .invert = Інверсія

# -----------------------------------
#         Toast notifications
# -----------------------------------
toast-dismiss-label = Закрити
toast-update-item-error = Не вдалося зберегти зміни. Ваші редагування були скасовані.
toast-update-annotation-error = Не вдалося оновити анотацію. Ваші зміни були скасовані.
toast-delete-annotation-error = Не вдалося видалити анотацію. Її було відновлено.

# -----------------------------------
#            Page Activity
# -----------------------------------
page-activity = Активність по сторінках
    .reading-number = Читання { $number }
    .select-reading = Вибрати прочитання
    .page = Сторінка { $page }
    .unread = не прочитано
    .visits = { $count ->
        [one] { $count } відвідування
        [few] { $count } відвідування
       *[other] { $count } відвідувань
    }
    .highlights = { $count ->
        [one] { $count } виділення
        [few] { $count } виділення
       *[other] { $count } виділень
    }
    .bookmarks = { $count ->
        [one] { $count } закладка
        [few] { $count } закладки
       *[other] { $count } закладок
    }
    .notes = { $count ->
        [one] { $count } нотатка
        [few] { $count } нотатки
       *[other] { $count } нотаток
    }
    .of = з { $total }
    .pages-read = прочитано сторінок
    .no-data = Для цього елемента відсутні дані про читання на рівні сторінок.
    .legend-bookmark = Закладка
    .legend-chapter = Розділ
    .info = Номери сторінок залежать від поточних параметрів форматування документа. На відміну від інших розділів KoShelf, це подання не підтримує синтетичне масштабування сторінок.
