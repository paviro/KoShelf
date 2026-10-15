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

# -----------------------------------
#             Settings
# -----------------------------------
appearance-setting = Оформление
theme-setting = Тема
    .description = Выберите внешний вид KoShelf. Авто следует системной теме.
    .option-auto = Авто
    .option-light = Светлая
    .option-dark = Темная
prefetch-setting = Предзагрузка ссылок
    .description = Предварительно загружайте страницы при наведении, фокусе или касании ссылок, чтобы навигация ощущалась быстрее.
    .option-enabled = Включено
    .option-disabled = Выключено
    .connection-note = Примечание: предзагрузка все равно автоматически отключается при ограниченном соединении (например, режим экономии трафика или более медленные мобильные сети).
language-setting = Язык
    .hint = Задаёт язык для всех переводов интерфейса.
region-setting = Регион
    .hint = Влияет на формат дат и чисел.
    .preview-date = Предпросмотр Дата
    .preview-number = Предпросмотр Число
    .majority-group = Регионы с языковым большинством
    .all-group = Все поддерживаемые регионы

# -----------------------------------
#         Authentication
# -----------------------------------
login = Войти
    .title = Войдите в { $site }
    .password = Пароль
    .submit = Войти
    .error = Неверный пароль
    .rate-limited = Слишком много попыток. Попробуйте снова чуть позже.
change-password = Сменить пароль
    .setting = Пароль
    .current = Текущий пароль
    .new = Новый пароль
    .confirm = Подтвердите пароль
    .changed = Пароль успешно изменен
    .too-short = Пароль должен содержать не менее { $min } символов
    .mismatch = Пароли не совпадают
    .incorrect = Неверный текущий пароль
    .current-placeholder = Введите текущий пароль
    .new-placeholder = Мин. 8 символов
    .new-hint = Должен содержать не менее 8 символов
    .confirm-placeholder = Повторите новый пароль
session-management =
    .setting = Сеансы
    .current = Текущий сеанс
    .this-device = Это устройство
    .last-active = Последняя активность
    .revoke = Отозвать
    .revoke-confirm = Отозвать этот сеанс?
    .revoked = Сеанс отозван
    .device-info = { $browser } на { $os }
    .logout = Выйти

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
    .on-hold-aria = { filter.aria-label } - Текущий: { filter.on-hold }
no-books-found = Книги не найдены
no-books-match = Нет книг, соответствующих вашему поиску или фильтру.
try-adjusting = Попробуйте изменить критерии поиска или фильтра
status =
    .reading = Читаю сейчас
    .reading-short = Читаю
    .on-hold = Отложено
    .completed = Прочитано
    .completed-short = Завершено
    .unread = Не прочитано
    .abandoned = Заброшено
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
statistics-from-koreader = Статистика из сессий чтения KoReader.
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
    .go-to-list = Открыть {$collection}
    .crash-title = Что-то пошло не так
    .crash-description = При отображении этой страницы произошла непредвиденная ошибка.
    .crash-report = Сообщить о проблеме

# Navigation and sorting
sort-order =
    .aria-label-toggle = Переключить порядок сортировки
    .newest-first = { sort-order.aria-label-toggle } - Текущий: Сначала новые
    .oldest-first = { sort-order.aria-label-toggle } - Текущий: Сначала старые
    .page-ascending = { sort-order.aria-label-toggle } - Текущий: по номеру страницы по возрастанию
    .page-descending = { sort-order.aria-label-toggle } - Текущий: по номеру страницы по убыванию
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

# -----------------------------------
#              Reader
# -----------------------------------
reader =
    .title = Читалка
    .loading = Загрузка книги…
    .previous-page = Предыдущая страница
    .next-page = Следующая страница
    .contents = Содержание
open-at-annotation = Открыть на аннотации
open-in-reader = Открыть
    .aria-label = Открыть в читалке
reader-settings =
    .label = Настройки
    .aria-label = Настройки отображения
    .typography = Типографика
    .margins = Отступы
reader-font-size = Размер шрифта
    .decrease-aria = Уменьшить размер шрифта
    .increase-aria = Увеличить размер шрифта
reader-line-spacing = Межстрочный интервал
    .decrease-aria = Уменьшить межстрочный интервал
    .increase-aria = Увеличить межстрочный интервал
reader-word-spacing = Межсловный интервал
    .decrease-aria = Уменьшить межсловный интервал
    .increase-aria = Увеличить межсловный интервал
reader-hyphenation = Переносы
reader-floating-punctuation = Висячая пунктуация
reader-embedded-fonts = Встроенные шрифты
reader-left-margin = Левый отступ
    .decrease-aria = Уменьшить левый отступ
    .increase-aria = Увеличить левый отступ
reader-right-margin = Правый отступ
    .decrease-aria = Уменьшить правый отступ
    .increase-aria = Увеличить правый отступ
reader-top-margin = Верхний отступ
    .decrease-aria = Уменьшить верхний отступ
    .increase-aria = Увеличить верхний отступ
reader-bottom-margin = Нижний отступ
    .decrease-aria = Уменьшить нижний отступ
    .increase-aria = Увеличить нижний отступ
reader-mode =
    .auto = Книга
    .on = Вкл
    .off = Выкл
reader-reset =
    .book = Настройки книги
    .book-aria = Вернуть синхронизированные настройки отображения книги
    .defaults = Сбросить по умолчанию
    .defaults-aria = Сбросить настройки отображения читалки по умолчанию
reader-drawer =
    .label = Навигация
    .aria-label = Панель навигации по книге
reader-no-toc = Оглавление недоступно
    .description = В этом файле нет маркеров глав.
reader-no-highlights = Пока нет выделений
    .description = Выделения, которые вы добавите в KoReader, появятся здесь.
reader-no-bookmarks = Пока нет закладок
    .description = Закладки, которые вы добавите в KoReader, появятся здесь.
select-month = Выбрать месяц

# -----------------------------------
#           Time & Dates
# -----------------------------------
weekday =
    .mon = Пн
    .tue = Вт
    .wed = Ср
    .thu = Чт
    .fri = Пт
    .sat = Сб
    .sun = Вс

units =
    .w = нед
    .d = д
    .h = ч
    .m = мин
    .s = s
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

# -----------------------------------
#         Writeback / Editing
# -----------------------------------
edit-warning =
    .title = Перед редактированием
    .body = Убедитесь, что книга закрыта в KoReader и была закрыта достаточно долго, чтобы синхронизация файлов (например, Syncthing) завершилась. KoReader перезаписывает файл метаданных, когда книга открыта. Если вы сохраните изменения здесь, пока книга ещё открыта (или до того, как последний sidecar-файл синхронизировался) ваши изменения будут потеряны.
    .dismiss = Больше не показывать это предупреждение
    .understood = Понятно
edit =
    .aria-label = Редактировать
save = Сохранить
cancel = Отмена
delete = Удалить
no-review-available = Рецензия отсутствует
    .hint-edit = Используйте кнопку редактирования, чтобы добавить рецензию и оценку.
    .hint-readonly = Рецензии можно добавлять непосредственно в KoReader.
add-review = Добавить рецензию
delete-review = Удалить рецензию
add-note = Добавить заметку
edit-note = Редактировать заметку
delete-note = Удалить заметку
delete-highlight = Удалить выделение
delete-highlight-and-note = Удалить выделение и заметку
delete-bookmark = Удалить закладку
highlight-color =
    .aria-label = Цвет
highlight-drawer =
    .aria-label = Стиль
    .lighten = Выделение
    .underscore = Подчёркивание
    .strikeout = Зачёркивание
    .invert = Инверсия

# -----------------------------------
#         Toast notifications
# -----------------------------------
toast-dismiss-label = Закрыть
toast-update-item-error = Не удалось сохранить изменения
    .subtitle = Ваши правки были отменены.
toast-update-annotation-error = Не удалось обновить аннотацию
    .subtitle = Ваши изменения были отменены.
toast-delete-annotation-error = Не удалось удалить аннотацию
    .subtitle = Она была восстановлена.

# -----------------------------------
#            Page Activity
# -----------------------------------
page-activity = Активность по страницам
    .select-reading = Выбрать прочтение
    .page = Страница { $page }
    .unread = не прочитано
    .visits = { $count ->
        [one] { $count } посещение
        [few] { $count } посещения
       *[other] { $count } посещений
    }
    .highlights = { $count ->
        [one] { $count } выделение
        [few] { $count } выделения
       *[other] { $count } выделений
    }
    .bookmarks = { $count ->
        [one] { $count } закладка
        [few] { $count } закладки
       *[other] { $count } закладок
    }
    .notes = { $count ->
        [one] { $count } заметка
        [few] { $count } заметки
       *[other] { $count } заметок
    }
    .of = из { $total }
    .pages-read = прочитано страниц
    .no-data = Для этого элемента отсутствуют данные о чтении на уровне страниц.
    .legend-bookmark = Закладка
    .legend-chapter = Глава
    .info = Номера страниц зависят от текущих параметров форматирования документа. В отличие от других разделов KoShelf, это представление не поддерживает синтетическое масштабирование страниц.

metadata-edit-stale = Метаданные изменились во время редактирования. Черновик сохранён. Закройте редактор и проверьте актуальные данные перед повторным редактированием.
metadata-backup-edit = Эти данные восстановлены из резервной копии KOReader. При сохранении изменений будет восстановлен основной файл метаданных.

library-archived = В архиве
library-available = Доступно
library-all = Все
library-availability = Доступность
library-remove = Удалить архив
library-remove-book-body = Эта книга больше не будет отображаться в вашей библиотеке.
library-remove-comic-body = Этот комикс больше не будет отображаться в вашей библиотеке.
library-remove-immediate = KoShelf сразу удалит архив.
library-remove-forever = KoShelf будет бессрочно хранить архив в своей папке данных.
library-remove-days = { $days ->
    [one] KoShelf будет хранить архив в своей папке данных { $days } день, а затем удалит его.
    [few] KoShelf будет хранить архив в своей папке данных { $days } дня, а затем удалит его.
    [many] KoShelf будет хранить архив в своей папке данных { $days } дней, а затем удалит его.
   *[other] KoShelf будет хранить архив в своей папке данных { $days } дня, а затем удалит его.
}
library-remove-error = Не удалось убрать из библиотеки. Обновите страницу и попробуйте снова.
library-backup-pending = Изменения сохранены в исходном файле. Резервное копирование ещё не завершено; попытка будет повторена.

library-reading-status = Статус чтения

# Custom metadata and covers
custom-edit-title = Изменить сведения
custom-tab-status = Статус
custom-metadata = Метаданные
custom-cover = Обложка
custom-override = Переопределено
custom-reset = Сбросить
custom-reset-original = Восстановить исходное
custom-metadata-reset-pending = Исходные метаданные будут восстановлены при сохранении.
custom-cover-reset-pending = Исходная обложка будет восстановлена при сохранении.
custom-cover-choose = Перетащите изображение сюда или нажмите, чтобы выбрать
koreader-cache-notice =
    .title = Изменения могут появиться не сразу
    .body = KOReader и некоторые его плагины кешируют сведения о книгах, поэтому изменение может появиться на устройстве не сразу.
    .step-refresh = В файловом браузере KOReader нажмите и удерживайте книгу и выберите «Обновить кешированную информацию о книге».
    .step-bookshelf = Пользуетесь плагином Bookshelf? Нажмите и удерживайте обложку, откройте вкладку «Изменить» и выберите «Обновить метаданные».
    .step-restart = Если изменение так и не появилось, перезапустите KOReader.
    .step-repeat = Всё ещё не видно? Повторите эти шаги ещё несколько раз: иногда KOReader и его плагинам нужно несколько попыток, чтобы подхватить изменение.
    .dismiss = Больше не показывать
    .ok = Понятно
custom-issues = Сведения об источнике
custom-cover-preview = Предпросмотр обложки
custom-series-index-invalid = Номер в серии должен быть числом, например 2 или 2.5.
custom-lists-help = Укажите каждого автора или ключевое слово на отдельной строке.
custom-edit-stale = Источник изменился. Отмените изменения и откройте редактор заново.
custom-edit-unavailable = Редактирование недоступно
custom-field-title = Название
custom-field-authors = Авторы
custom-field-series = Серия
custom-field-series_index = Номер в серии
custom-field-language = Язык
custom-field-keywords = Ключевые слова
custom-field-description = Описание
custom-reason-writeback_disabled = Запись отключена
custom-reason-archived = Архивный элемент
custom-reason-inaccessible_custom_files = Не удалось прочитать пользовательские файлы
custom-saving = Сохранение…


library-metadata = Метаданные
