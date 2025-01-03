# pike (perfectly incomplex konsole editor) - szkielet kodu

#### Maksym Bieńkowski, Jędrzej Grabski

![Linux](https://github.com/pike-text-editor/pike/actions/workflows/linux-ci.yaml/badge.svg)
![Windows](https://github.com/pike-text-editor/pike/actions/workflows/windows-ci.yaml/badge.svg)

Branch zawiera szkielet kodu z ogólnymi, niewypełnionymi sygnaturami
oraz definicjami obiektów. Szybki overview modułów:

- `app` - zawiera frontendową porcję projektu, obiekt reprezentujący jego stan,
  funkcjonalność związaną z obsługą zdarzeń i rysowaniem ekranu.
- `pike` - reprezentacja backendowej części projektu, zarządza otwartymi buforami, trzyma
  informację o cwd, wchodzi w interakcję z systemem plików
- `config` - zawiera obiekt reprezentujący konfigurację użytkownika, który ładowany
  będzie z pliku konfiguracyjnego. Póki co, jedynymi wspieranymi ustawieniami są
  skróty klawiszowe, prawdopodobnie się to zmieni na przestrzeni czasu.
- `operations` - enumeracja reprezentująca mapowalne na skróty klawiszowe
  operacje i tworzenie ich na podstawie stringów zawartych w pliku
- `ui` - moduł zawierający reużywalne komponenty frontendowe, na ten moment
  ograniczony do obiektu `Picker`, na podstawie którego ma działać m.in.
  wybór pliku do otwarcia przy wyszukiwaniu po nazwie

## Użycie

### Przy pomocy `just`

`just lint` sprawdza kod za pomocą `cargo clippy`.

`just build` buduje projekt w trybie release.

`just test` uruchamia testy z włączonymi wszystkimi funkcjami.

`just windows-build` buduje projekt dla Windowsa za pomocą `cross`.

- wymaga [cross](https://github.com/cross-rs/cross).

`just windows-test` uruchamia testy dla Windowsa za pomocą `cross`.

- wymaga [cross](https://github.com/cross-rs/cross).

`just cov` generuje raport pokrycia testami w formacie HTML.

- wymaga [cargo-binutils](https://github.com/rust-embedded/cargo-binutils) oraz [grcov](https://github.com/mozilla/grcov)

### Bez `just`

`cargo run` uruchamia projekt

`cargo test` uruchamia testy w projekcie

`cargo doc -p pike` generuje dokumentację na podstawie komentarzy

#### Kompilacja dla windows

Do kompilacji skrośnej na Windowsa używamy [cross](https://github.com/cross-rs/cross).

Wymaga on dostępu do dockera lub podmana na hoście,
więcej w [dokumentacji](https://github.com/cross-rs/cross?tab=readme-ov-file#usage).

`cross build --target=x86_64-pc-windows-gnu` kompiluje projekt na windows toolchainem gnu

`cross test --target=x86_64-pc-windows-gnu` uruchamia testy dla windowsa w skonteneryzowanym środowisku

#### Narzędzia

##### Formatter

Używanym w projekcie formatterem jest [rustfmt](https://github.com/rust-lang/rustfmt).

Instalacja:

```bash
rustup update
rustup component add rustfmt
```

Uruchomienie:

```bash
cargo fmt           # formatuje pliki in-place
cargo fmt --check   # nie formatuje plików in-place
```

##### Linter

Używanym linterem jest [clippy](https://github.com/rust-lang/rust-clippy)

Instalacja:

```bash
rustup update
rustup component add clippy
```

Uruchomienie:

```bash
cargo clippy        # bez aplikowania sugestii
cargo clippy --fix  # aplikuje sugestie
```

Oba narzędzia powinny być dostępne w domyślnej instalacji Rusta
