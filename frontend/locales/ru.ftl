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
settings = Настройки
github = GitHub
reading-companion = Помощник чтения
# Used in footer/sidebar for update time
last-updated = Последнее обновление
view-details = Подробнее
appearance-setting = Оформление
theme-setting = Тема
theme-setting-description = Выберите внешний вид KoShelf. Авто следует системной теме.
theme-option-auto = Авто
theme-option-light = Светлая
theme-option-dark = Темная
prefetch-setting = Предзагрузка ссылок
prefetch-setting-description = Предварительно загружайте страницы при наведении, фокусе или касании ссылок, чтобы навигация ощущалась быстрее.
prefetch-option-enabled = Включено
prefetch-option-disabled = Выключено
prefetch-setting-connection-note = Примечание: предзагрузка все равно автоматически отключается при ограниченном соединении (например, режим экономии трафика или более медленные мобильные сети).
language-setting = Язык
region-setting = Регион
language-setting-hint = Задаёт язык для всех переводов интерфейса.
region-setting-hint = Влияет на формат дат и чисел.
preview-date = Предпросмотр Дата
preview-number = Предпросмотр Число
region-setting-majority-group = Регионы с языковым большинством
region-setting-all-group = Все поддерживаемые регионы
login = Войти
login-title = Войдите в { $site }
login-password = Пароль
login-submit = Войти
login-error = Неверный пароль
login-rate-limited = Слишком много попыток. Попробуйте снова чуть позже.
password-setting = Пароль
change-password = Сменить пароль
current-password = Текущий пароль
new-password = Новый пароль
confirm-password = Подтвердите пароль
password-changed = Пароль успешно изменен
password-too-short = Пароль должен содержать не менее { $min } символов
password-mismatch = Пароли не совпадают
incorrect-password = Неверный текущий пароль
sessions-setting = Сеансы
current-session = Текущий сеанс
this-device = Это устройство
last-active = Последняя активность
revoke-session = Отозвать
revoke-session-confirm = Отозвать этот сеанс?
session-revoked = Сеанс отозван
session-device-info = { $browser } на { $os }
logout = Выйти
current-password-placeholder = Введите текущий пароль
new-password-placeholder = Мин. 8 символов
new-password-hint = Должен содержать не менее 8 символов
confirm-password-placeholder = Повторите новый пароль

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
notes-label = { $count ->
    [one] Заметка
    [few] Заметки
    [many] Заметок
   *[other] Заметок
}
bookmarks = Закладки
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
yearly-statistics = Годовая статистика
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
average-time-day = Среднее время/день
average-pages-day = Средние страницы/день
most-pages-in-day = Больше всего страниц за день
longest-daily-reading = Самое длинное чтение за день
reading-completions = Завершённые чтения
completed-books = Завершённые книги
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
    .details = 1500 x 1500 — Квадрат 1:1
recap-banner = Баннер
    .details = 2400 x 1260 — Горизонтальный 2:1
best-month = Лучший месяц
active-days = { $count ->
    [one] Активный день
    [few] Активных дня
    [many] Активных дней
   *[other] Активных дней
}
active-days-tooltip = { $count ->
    [one] активный день
    [few] активных дня
    [many] активных дней
   *[other] активных дней
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
error-state =
    .title = Что-то пошло не так
    .description =
        Не удалось загрузить данные.
        Пожалуйста, попробуйте ещё раз.
    .not-found-title = Не найдено
    .not-found-description = Страница, которую вы ищете, не существует или была удалена.
    .connection-title = Ошибка подключения
    .connection-description =
        Не удалось связаться с сервером.
        Проверьте подключение и попробуйте снова.
    .file-unavailable-title = Файл книги недоступен
    .file-unavailable-description = Данные книги найдены, но файл книги отсутствует.
    .retry = Попробовать снова

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
go-back =
    .aria-label = Назад
open-reader-aria = Открыть в читалке
reader-title = Читалка
reader-loading = Загрузка книги…
reader-previous-page = Предыдущая страница
reader-next-page = Следующая страница
open-at-annotation = Открыть на аннотации
reader-contents = Содержание
reader-settings-aria = Настройки отображения
reader-section-text = Текст
reader-section-typography = Типографика
reader-section-margins = Отступы
reader-font-size = Размер шрифта
reader-font-size-decrease-aria = Уменьшить размер шрифта
reader-font-size-increase-aria = Увеличить размер шрифта
reader-line-spacing = Межстрочный интервал
reader-line-spacing-decrease-aria = Уменьшить межстрочный интервал
reader-line-spacing-increase-aria = Увеличить межстрочный интервал
reader-word-spacing = Межсловный интервал
reader-word-spacing-decrease-aria = Уменьшить межсловный интервал
reader-word-spacing-increase-aria = Увеличить межсловный интервал
reader-hyphenation = Переносы
reader-floating-punctuation = Висячая пунктуация
reader-embedded-fonts = Встроенные шрифты
reader-left-margin = Левый отступ
reader-left-margin-decrease-aria = Уменьшить левый отступ
reader-left-margin-increase-aria = Увеличить левый отступ
reader-right-margin = Правый отступ
reader-right-margin-decrease-aria = Уменьшить правый отступ
reader-right-margin-increase-aria = Увеличить правый отступ
reader-top-margin = Верхний отступ
reader-top-margin-decrease-aria = Уменьшить верхний отступ
reader-top-margin-increase-aria = Увеличить верхний отступ
reader-bottom-margin = Нижний отступ
reader-bottom-margin-decrease-aria = Уменьшить нижний отступ
reader-bottom-margin-increase-aria = Увеличить нижний отступ
reader-mode-auto = Книга
reader-mode-on = Вкл
reader-mode-off = Выкл
reader-reset-book = Настройки книги
reader-reset-book-aria = Вернуть синхронизированные настройки отображения книги
reader-reset-defaults = Сбросить по умолчанию
reader-reset-defaults-aria = Сбросить настройки отображения читалки по умолчанию
reader-drawer-aria = Панель навигации по книге
reader-no-toc = Оглавление недоступно
reader-no-toc-description = В этом файле нет маркеров глав.
reader-no-highlights = Пока нет выделений
reader-no-highlights-description = Выделения, которые вы добавите в KoReader, появятся здесь.
reader-no-bookmarks = Пока нет закладок
reader-no-bookmarks-description = Закладки, которые вы добавите в KoReader, появятся здесь.
open-in-reader = Открыть
select-month = Выбрать месяц

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
    .tue = Вт
    .wed = Ср
    .thu = Чт
    .fri = Пт
    .sat = Сб
    .sun = Вс

# Time units: w=weeks, d=days, h=hours, m=minutes
units =
    .w = нед
    .d = д
    .h = ч
    .m = мин
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
