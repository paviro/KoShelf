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
settings = Configurações
github = GitHub
reading-companion = Companheiro de Leitura
# Used in footer/sidebar for update time
last-updated = Última atualização
view-details = Ver Detalhes

# -----------------------------------
#             Settings
# -----------------------------------
appearance-setting = Aparência
theme-setting = Tema
    .description = Escolha como o KoShelf deve aparecer. Automático segue a preferência do seu sistema.
    .option-auto = Automático
    .option-light = Claro
    .option-dark = Escuro
prefetch-setting = Pré-carregamento de links
    .description = Pré-carrega páginas ao passar o mouse, focar ou tocar em links para deixar a navegação mais rápida.
    .option-enabled = Ativado
    .option-disabled = Desativado
    .connection-note = Observação: o pré-carregamento ainda é ignorado automaticamente quando a conexão está limitada (por exemplo, Economia de dados ou redes móveis mais lentas).
language-setting = Idioma
    .hint = Define o idioma utilizado para todas as traduções da interface.
region-setting = Região
    .hint = Afeta a formatação de datas e números.
    .preview-date = Pré-visualização Data
    .preview-number = Pré-visualização Número
    .majority-group = Regiões de maioria linguística
    .all-group = Todas as regiões compatíveis

# -----------------------------------
#         Authentication
# -----------------------------------
login = Iniciar sessão
    .title = Entre no { $site }
    .password = Palavra-passe
    .submit = Entrar
    .error = Palavra-passe inválida
    .rate-limited = Muitas tentativas. Tente novamente em breve.
change-password = Alterar palavra-passe
    .setting = Palavra-passe
    .current = Palavra-passe atual
    .new = Nova palavra-passe
    .confirm = Confirmar palavra-passe
    .changed = Palavra-passe alterada com sucesso
    .too-short = A palavra-passe deve ter pelo menos { $min } caracteres
    .mismatch = As palavras-passe não coincidem
    .incorrect = Palavra-passe atual incorreta
    .current-placeholder = Insira a palavra-passe atual
    .new-placeholder = Mín. 8 caracteres
    .new-hint = Deve ter pelo menos 8 caracteres
    .confirm-placeholder = Reinsira a nova palavra-passe
session-management =
    .setting = Sessões
    .current = Sessão atual
    .this-device = Este dispositivo
    .last-active = Última atividade
    .revoke = Revogar
    .revoke-confirm = Revogar esta sessão?
    .revoked = Sessão revogada
    .device-info = { $browser } em { $os }
    .logout = Terminar sessão

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
    .reading-short = A ler
    .on-hold = Em Espera
    .completed = Concluído
    .completed-short = Concluído
    .unread = Não lido
    .abandoned = Abandonado
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
notes-label = { $count ->
    [one] Nota
   *[other] Notas
}
bookmarks = Marcadores
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
yearly-statistics = Estatísticas Anuais
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
average-time-day = Tempo Médio/Dia
average-pages-day = Média de Páginas/Dia
most-pages-in-day = Mais Páginas em um Dia
longest-daily-reading = Maior Leitura Diária
reading-completions = Leituras Concluídas
completed-books = Livros Concluídos
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
active-days-tooltip = { $count ->
    [one] dia ativo
   *[other] dias ativos
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
error-state =
    .title = Algo deu errado
    .description =
        Não foi possível carregar os dados.
        Por favor, tente novamente.
    .not-found-title = Não encontrado
    .not-found-description = A página que você procura não existe ou foi removida.
    .connection-title = Falha na conexão
    .connection-description =
        Não foi possível contactar o servidor.
        Verifique sua conexão e tente novamente.
    .file-unavailable-title = Arquivo do livro indisponível
    .file-unavailable-description = Os detalhes do livro foram encontrados, mas o arquivo do livro está ausente.
    .retry = Tentar novamente

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
go-back =
    .aria-label = Voltar

# -----------------------------------
#              Reader
# -----------------------------------
reader =
    .title = Leitor
    .loading = A carregar livro…
    .previous-page = Página anterior
    .next-page = Próxima página
    .contents = Conteúdo
open-at-annotation = Abrir na anotação
open-in-reader = Abrir
    .aria-label = Abrir no leitor
reader-settings =
    .aria-label = Configurações de exibição
    .typography = Tipografia
    .margins = Margens
reader-font-size = Tamanho da fonte
    .decrease-aria = Diminuir tamanho da fonte
    .increase-aria = Aumentar tamanho da fonte
reader-line-spacing = Espaçamento entre linhas
    .decrease-aria = Diminuir espaçamento entre linhas
    .increase-aria = Aumentar espaçamento entre linhas
reader-word-spacing = Espaçamento entre palavras
    .decrease-aria = Diminuir espaçamento entre palavras
    .increase-aria = Aumentar espaçamento entre palavras
reader-hyphenation = Hifenização
reader-floating-punctuation = Pontuação flutuante
reader-embedded-fonts = Fontes incorporadas
reader-left-margin = Margem esquerda
    .decrease-aria = Diminuir margem esquerda
    .increase-aria = Aumentar margem esquerda
reader-right-margin = Margem direita
    .decrease-aria = Diminuir margem direita
    .increase-aria = Aumentar margem direita
reader-top-margin = Margem superior
    .decrease-aria = Diminuir margem superior
    .increase-aria = Aumentar margem superior
reader-bottom-margin = Margem inferior
    .decrease-aria = Diminuir margem inferior
    .increase-aria = Aumentar margem inferior
reader-mode =
    .auto = Livro
    .on = Ativado
    .off = Desativado
reader-reset =
    .book = Usar configurações do livro
    .book-aria = Restaurar para as configurações de exibição sincronizadas do livro
    .defaults = Restaurar padrões
    .defaults-aria = Restaurar as configurações de exibição do leitor para os padrões
reader-drawer =
    .aria-label = Painel de navegação do livro
reader-no-toc = Nenhum sumário disponível
    .description = Este ficheiro não inclui marcadores de capítulo.
reader-no-highlights = Ainda não há destaques
    .description = Os destaques que adicionar no KoReader aparecerão aqui.
reader-no-bookmarks = Ainda não há marcadores
    .description = Os marcadores que adicionar no KoReader aparecerão aqui.
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
    .tue = Ter
    .wed = Qua
    .thu = Qui
    .fri = Sex
    .sat = Sáb
    .sun = Dom

# Time units: w=weeks, d=days, h=hours, m=minutes
units =
    .w = sem
    .d = d
    .h = h
    .m = m
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

# -----------------------------------
#         Writeback / Editing
# -----------------------------------
edit-warning =
    .title = Antes de editar
    .body = Certifique-se de que o livro está fechado no KoReader e que ficou fechado tempo suficiente para que qualquer sincronização de ficheiros (por exemplo, Syncthing) tenha sido concluída. O KoReader sobrescreve o ficheiro de metadados quando um livro está aberto. Se guardar alterações aqui enquanto o livro ainda estiver aberto — ou antes de o último ficheiro sidecar ter sido sincronizado — as suas alterações serão perdidas.
    .dismiss = Não mostrar este aviso novamente
    .understood = Entendido
edit =
    .aria-label = Editar
save = Guardar
cancel = Cancelar
delete = Eliminar
no-review-available = Sem avaliação disponível
    .hint-edit = Use o botão de edição para adicionar a sua avaliação e classificação.
    .hint-readonly = As avaliações podem ser adicionadas diretamente no KoReader.
add-review = Adicionar avaliação
delete-review = Eliminar avaliação
add-note = Adicionar nota
edit-note = Editar nota
delete-note = Eliminar nota
delete-highlight = Eliminar destaque
delete-highlight-and-note = Eliminar destaque e nota
delete-bookmark = Eliminar marcador
change-status = Alterar estado
highlight-color =
    .aria-label = Cor do destaque
highlight-drawer =
    .aria-label = Estilo de destaque
    .lighten = Destaque
    .underscore = Sublinhado
    .strikeout = Riscado
    .invert = Invertido

# -----------------------------------
#         Toast notifications
# -----------------------------------
toast-dismiss-label = Dispensar
toast-update-item-error = Não foi possível guardar as alterações. As suas edições foram revertidas.
toast-update-annotation-error = Não foi possível atualizar a anotação. As suas alterações foram revertidas.
toast-delete-annotation-error = Não foi possível eliminar a anotação. Foi restaurada.
