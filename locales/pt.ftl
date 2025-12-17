# ===========================================
#      Portuguese (pt) — Base Language File
# ===========================================
# This is the base translation file for Portuguese, using Brazilian Portuguese (pt_BR).
# Regional variants (e.g., pt_PT.ftl) should only override keys that differ.
#
# LOCALE HIERARCHY:
#   1. Regional variant (e.g., pt_PT.ftl) — sparse, only differences
#   2. Base language (this file) — complete translations
#   3. English fallback (en.ftl) — ultimate fallback for all languages
#
# NEW KEYS: Always add new translation keys to en.ftl first, then other bases.

# Machine-readable metadata (used by --list-languages)
-lang-code = pt
-lang-name = Português (Brasil)
-lang-dialect = pt_BR

# -----------------------------------
#           Navigation & Shared
# -----------------------------------
books = Livros
comics = Quadrinhos
statistics = Estatísticas
calendar = Calendário
recap = Retrospectiva
github = GitHub
loading = Carregando...
reload = Recarregar
new-version-available = Nova versão disponível
tap-to-reload = Toque para recarregar
reading-companion = Companheiro de Leitura
# Used in footer/sidebar for update time
last-updated = Última atualização
view-details = Ver Detalhes

# -----------------------------------
#        Book List & Library
# -----------------------------------
search-placeholder = Buscar livro, autor, série...
filter =
    .aria-label = Filtrar livros
    .all = Todos
    .all-aria = { filter.aria-label } - Atual: { filter.all }
    .reading = { status.reading }
    .reading-aria = { filter.aria-label } - Atual: { filter.reading }
    .completed = { status.completed }
    .completed-aria = { filter.aria-label } - Atual: { filter.completed }
    .unread = { status.unread }
    .unread-aria = { filter.aria-label } - Atual: { filter.unread }
    .on-hold = { status.on-hold }
    .on-hold-aria = { filter.aria-label } - Atual: { filter.on-hold }
no-books-found = Nenhum livro encontrado
no-books-match = Nenhum livro corresponde aos seus critérios de busca ou filtro.
try-adjusting = Tente ajustar sua busca ou filtros
status =
    .reading = Lendo Atualmente
    .on-hold = Em Espera
    .completed = Concluído
    .unread = Não lido
book-label = { $count ->
    [one] Livro
   *[other] Livros
}
comic-label = { $count ->
    [one] Banda desenhada
   *[other] Bandas desenhadas
}
books-finished = { $count ->
    [one] { book-label } Terminado
   *[other] { book-label } Terminados
}
comics-finished = { $count ->
    [one] { comic-label } Terminada
   *[other] { comic-label } Terminadas
}
unknown-book = Livro Desconhecido
unknown-author = Autor Desconhecido
by = por
book-overview = Visão Geral
comic-overview = Visão Geral da Banda Desenhada

# -----------------------------------
#            Book Details
# -----------------------------------
description = Descrição
publisher = Editora
series = Série
genres = Gêneros
language = Idioma
book-identifiers = Identificadores
my-review = Minha Resenha
my-note = Minha Nota
highlights = Destaques
highlights-label = { $count ->
    [one] Destaque
   *[other] Destaques
}
notes = Notas
notes-label = { $count ->
    [one] Nota
   *[other] Notas
}
bookmarks = Marcadores
page = Página
page-bookmark = Marcador de Página
bookmark-anchor = Âncora do Marcador
highlights-quotes = Destaques & Citações
additional-information = Informações Adicionais
reading-progress = Progresso de Leitura
page-number = Página { $count }
last-read = Última Leitura
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
reading-statistics = Estatísticas de Leitura
overall-statistics = Estatísticas Gerais
weekly-statistics = Estatísticas Semanais
total-read-time = Tempo Total de Leitura
total-pages-read = Total de Páginas Lidas
pages-per-hour = Páginas/Hora
# Abbreviation for Pages Per Hour
pph-abbreviation = pph
reading-sessions-label = { $count ->
    [one] Sessão de Leitura
   *[other] Sessões de Leitura
}
session =
    .longest = Sessão Mais Longa
    .average = Sessão Média
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /sessão média
streak =
    .current = Sequência Atual
    .longest = Maior Sequência
reading-streak = Sequência de Leitura
days-read = Dias Lidos
weekly-reading-time = Tempo de Leitura Semanal
weekly-pages-read = Páginas Lidas na Semana
average-time-day = Tempo Médio/Dia
average-pages-day = Média de Páginas/Dia
most-pages-in-day = Mais Páginas em um Dia
longest-daily-reading = Maior Leitura Diária
reading-completions = Leituras Concluídas
statistics-from-koreader = Estatísticas das sessões de leitura do KoReader
reading-time = Tempo de Leitura
pages-read = Páginas Lidas
units-days = { $count ->
    [one] { $count } dia
   *[other] { $count } dias
}
units-sessions = { $count ->
    [one] { $count } sessão
   *[other] { $count } sessões
}

# -----------------------------------
#               Recap
# -----------------------------------
my-reading-recap = Minha Retrospectiva KoShelf
share = Compartilhar
    .recap-label = Compartilhar Imagem
download = Baixar
    .recap-label = Baixar Imagem
recap-story = Story
    .details = 1260 x 2240 — Vertical 9:16
recap-square = Quadrado
    .details = 1500 x 1500 — Quadrado 1:1
recap-banner = Banner
    .details = 2400 x 1260 — Horizontal 2:1
best-month = Melhor Mês
active-days = { $count ->
    [one] Dia Ativo
   *[other] Dias Ativos
}
toggle =
    .hide = Ocultar
    .show = Mostrar
less = Menos
more = Mais
period = Período
sessions = Sessões
yearly-summary = Resumo Anual { $count }
recap-empty =
    .nothing-here = Nada aqui ainda
    .try-switching = Tente mudar o escopo ou o ano acima.
    .finish-reading = Termine de ler no KoReader para ver seu resumo.
    .info-question = Por que meu resumo não está aparecendo?
    .info-answer = O KoShelf usa estatísticas de leitura para detectar conclusões, o que permite o rastreamento de releituras. Apenas marcar um livro como "concluído" sem dados de leitura não fará com que ele apareça aqui.
stats-empty =
    .nothing-here = Nada aqui ainda
    .start-reading = Comece a ler com o KoReader para ver suas estatísticas aqui.
    .info-question = Como funciona o rastreamento de leitura?
    .info-answer = O KoReader rastreia automaticamente suas sessões de leitura, incluindo tempo gasto e páginas lidas. Sincronize seu banco de dados de estatísticas com o KoShelf para ver sua atividade visualizada aqui.

# Navigation and sorting
sort-order =
    .aria-label-toggle = Alternar ordem
    .newest-first = { sort-order.aria-label-toggle } - Atual: Mais Recentes Primeiro
    .oldest-first = { sort-order.aria-label-toggle } - Atual: Mais Antigos Primeiro
previous-month = 
    .aria-label = Mês anterior
next-month = 
    .aria-label = Próximo mês
search = 
    .aria-label = Buscar
close-search = 
    .aria-label = Fechar busca
close = Fechar
    .aria-label = Fechar
go-back =
    .aria-label = Voltar
select-month = Selecionar mês

# -----------------------------------
#           Time & Dates
# -----------------------------------
january = Janeiro
    .short = Jan
february = Fevereiro
    .short = Fev
march = Março
    .short = Mar
april = Abril
    .short = Abr
may = Maio
    .short = Mai
june = Junho
    .short = Jun
july = Julho
    .short = Jul
august = Agosto
    .short = Ago
september = Setembro
    .short = Set
october = Outubro
    .short = Out
november = Novembro
    .short = Nov
december = Dezembro
    .short = Dez

# Weekday abbreviations (only Mon/Thu/Sun for GitHub-style heatmap visualization)
weekday =
    .mon = Seg
    .thu = Qui
    .sun = Dom

# Chrono date/time format strings (use %B for full month, %b for short, etc.)
datetime =
    .full = %d de %B de %Y às %H:%M
    .short-current-year = %d de %b
    .short-with-year = %d %b %Y

# Time units: h=hours, m=minutes, d=days
units =
    .h = h
    .m = m
    .d = d
today = Hoje
of-the-year = do ano
of = de
last = Último

# Time unit labels (standalone word forms for displaying after numbers)
days_label = { $count ->
    [one] dia
   *[other] dias
}
hours_label = { $count ->
    [one] hora
   *[other] horas
}
minutes_label = { $count ->
    [one] minuto
   *[other] minutos
}
