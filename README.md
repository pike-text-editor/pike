
# pike (perfectly incomplex konsole editor) - szkielet kodu

#### Maksym Bieńkowski, Jędrzej Grabski

![Linux](https://github.com/pike-text-editor/pike/actions/workflows/linux-ci.yml/badge.svg)

![Windows](https://github.com/pike-text-editor/pike/actions/workflows/windows-ci.yml/badge.svg)

Branch zawiera szkielet kodu z ogólnymi, niewypełnionymi sygnaturami
oraz definicjami obiektów. Szybki overview modułów:

* `app` - zawiera frontendową porcję projektu, obiekt reprezentujący jego stan,
funkcjonalność związaną z obsługą zdarzeń i rysowaniem ekranu.
* `pike` - reprezentacja backendowej części projektu, zarządza otwartymi buforami, trzyma
informację o cwd, wchodzi w interakcję z systemem plików
* `config` - zawiera obiekt reprezentujący konfigurację użytkownika, który ładowany
będzie z pliku konfiguracyjnego. Póki co, jedynymi wspieranymi ustawieniami są
skróty klawiszowe, prawdopodobnie się to zmieni na przestrzeni czasu.
* `operations` - enumeracja reprezentująca mapowalne na skróty klawiszowe
operacje i tworzenie ich na podstawie stringów zawartych w pliku
* `ui` - moduł zawierający reużywalne komponenty frontendowe, na ten moment
ograniczony do obiektu `Picker`, na podstawie którego ma działać m.in.
wybór pliku do otwarcia przy wyszukiwaniu po nazwie

## Użycie

`cargo run` uruchamia projekt
`cargo test` uruchamia przykładowy przechodzący test
`cargo doc -p pike` generuje dokumentację na podstawie komentarzy

### Kompilacja dla windows

Do kompilacji skrośnej na Windowsa używamy [cross](https://github.com/cross-rs/cross).
Wymaga on dostępu do dockera lub podmana na hoście,
więcej w [dokumentacji](https://github.com/cross-rs/cross?tab=readme-ov-file#usage).

`cross build --target=x86_64-pc-windows-gnu` kompiluje projekt na windows toolchainem gnu
`cross test --target=x86_64-pc-windows-gnu` uruchamia testy dla windowsa w skonteneryzowanym środowisku
