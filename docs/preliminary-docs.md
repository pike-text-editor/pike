# pike (perfectly incomplex konsole editor) - dokumentacja wstępna

#### Maksym Bieńkowski, Jędrzej Grabski

## Opis zadania

Tematem naszego projektu jest implementacja prostego, funkcjonalnego terminalowego edytora tekstu, na wzór nano, z trochę innym
zestawem funkcjonalności. Wybraliśmy rusta, za backend będzie służył [crossterm](https://docs.rs/crossterm/latest/crossterm/), a frontend zrealizujemy przy pomocy
[ratatui](https://ratatui.rs/).

### Funkcjonalności

#### Edytowanie tekstu w stylu nano

* początkowo zawartość pliku reprezentowana jako string
* później przejście na gap buffer

#### Undo/redo

* historia liniowa, deque n ostatnich zmian
* zmiana składa się z offsetu od początku pliku, w którym wystąpiła, typu (dodaj/usuń) i tekstu, który został usunięty/dodany

#### Pasek statusowy

* nazwa pliku, filetype, ewentualnie podpowiedzi dotyczące skrótów klawiszowych

#### Plik konfiguracyjny

* format `toml`
* skróty klawiszowe
* ogólne ustawienia

#### Tworzenie plików i katalogów w cwd bez opuszczania edytora (zmiana edytowanego pliku)

* pod określonym keymapem, jak `:e` w vim

#### Wyszukiwanie w pliku

* Na bazie [ekosystemu rg](https://github.com/BurntSushi/ripgrep/tree/master/crates)

#### Wyszukiwanie tekstu w cwd

* funkcjonalność na wzór [Telescope.find_text]( "https://github.com/nvim-telescope/telescope.nvim" )
* na bazie ekosystemu rg

#### Wyszukiwanie plików w cwd

* funkcjonalność na wzór [Telescope.find_files]( "https://github.com/nvim-telescope/telescope.nvim" )
* [Walkdir](https://rust-lang-nursery.github.io/rust-cookbook/file/dir.html) + regex

#### Find and replace

* prawdopodobnie integracja z `sed` i możliwość wprowadzenia komendy na poziomie edytora, jak w vimie

#### Swapfile

* na start kopia bufora przechowywana w określonym miejscu
* przy uruchomieniu, edytor szuka pliku i rozwiązuje ewentualne konflikty

#### Podstawowy syntax highlighting

* rozwiązanie podobne do [kibi](https://github.com/ilai-deutel/kibi/blob/master/src/row.rs#L79) lub [kilo](https://github.com/antirez/kilo/blob/master/kilo.c#L364)

### Opcjonalne funkcjonalności, jeśli zostanie czas

* file picker drzewiasty/netrw
* integracja z autoformaterami ran on save, podpinane pod file type w pliku konfiguracyjnym
* wsparcie kilku trybów edytowania, uproszczony vim mode
* konfiguracja themes przez plik konfiguracyjny
* nagrywanie macro
* podstawowy autocompletion na podstawie wyrażeń zawartych w buforze, bez integracji z lsp
* sensowne api umożliwiające pisanie wtyczek, np. do integracji z build toolami, na wzór vima
* możliwość uruchamiania komend bez opuszczania edytora
