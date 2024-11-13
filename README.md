
# pike (perfectly incomplex konsole editor) - szkielet kodu

#### Maksym Bieńkowski, Jędrzej Grabski

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
