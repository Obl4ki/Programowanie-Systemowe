# Podstawy języka Rust

Ten dokument ma za zadanie pomóc w konfiguracji środowiska i naświetlić różnice pomiędzy językiem C, a Rustem. Nie jest to wyczerpujący poradnik.

Poradnik: https://doc.rust-lang.org/stable/book/

## Instalacja

https://rust-lang.org/learn/get-started

Do pisania kodu polecam Visual Studio Code
https://code.visualstudio.com

Z zainstalowanymi wtyczkami:

- Rust analyzer - dodaje obsługę języka do vscode
- Crates - dodaje podpowiedzi odnośnie pakietów języka (tzw. crate'ów) w plikach Cargo.toml
- Even Better Toml - dodaje obsługę do plików .toml

## Utworzenie pustego projektu

```bash
    cargo new <nazwa-projektu>
```

## Uruchamianie programów

W trybie debugowania (domyślnie z symbolami, bez optymalizacji)

```bash
cargo run
```

W trybie release (max. optymalizacja, brak symboli)

```bash
cargo run --release
```

## Zmienne

```rust
let x = 5;
let y = 5000000;
let z = 5_000_000;
// z zadeklarowanym typem
let x: usize = 5;
// dla doprecyzowania typu możliwe również dopisanie po wartości
let x = 5usize;
```

Zmienne domyślnie są immutable. Aby to zmienić należy dodać `mut`:

```rust
let mut x = 5;
x = 6;
assert!(x == 6);
```

Bez `mut` możliwa jest tylko redeklaracja zmiennej:

```rust
let x = 5;
let x = 6;
assert!(x == 6);
```

## Wyrażenia

Sama składni jest interpretowana trochę inaczej niż C. W języku funkcjonuje koncept wyrażeń (expression). Przydaje się to w nienadużywaniu `mut`. Jest możliwe np. taka technika:

```rust
let switch = true;
let x = if switch { 1 } else { 0 };
```

```rust
// Obie te rzeczy są równoważne
let x = { return 1; }
let x = { 1 }
```

Gdyby próbować wykonać bezpośrednią kalkę z C, to wyglądałoby to tak:

```rust
let switch = true
let mut x;

if switch {
    x = 1;
} else {
    x = 2;
}
```

Problem z tym jest taki, że po wyrażeniu x jest mutable, co może być źródłem błędu w innym miejscu w kodzie. Ponadto wyrażenia muszą być wyczerpujące (bez klauzuli else pojawi się compilation error).

## Strukury

```rust
struct Point {
    // [visibility] name: Type
    x: f64,
    y: f64,
    pub name: String
}
```

## Printowanie

```rust
let x = 5;
println!("Zmienna = {}", x); // w trybie Display
println!("Zmienna = {:?}", x); // w trybie Debug

```

W przypadku printowania własnych struktur dana struktura musi mieć zaimplementowany `trait`. Najprościej mówiąc `trait` to nieco rozszerzony mechanizm interfejsów (C#, Java) i klas abstrakcyjnych (C++), który jest jednym z głównych mechanizmów polimorficznych.

Aby móc coś wyprintować w trybie debugowania (flaga :?) potrzebny jest na tym trait `Debug` (std::fmt::Debug). Analogicznie sytuacja wygląda z traitem `Display`.

Debug określa, że struktury i enumy mogą się przetwarzać na tekst (np. do wyprintowania) na potrzeby programisty, i na tym się skupimy. Aby automatycznie zaimplementować trait debug dla naszej struktury, możemy napisać:

```rust
#[derive(Debug)]
//^^^^^^^^^^^^
struct Point {
    x: f64,
    y: f64
}
```

Makro derive samo zaimplementuje odpowiednią logikę, i od tej pory można wywołać:

```rust
let p1 = Point { x: 5., y: 6.5 };
println!("{:?}", p1);
```

## Iteracja

Jest wiele różnych metod na iterację. Poniższe funkcje implementują sumę liczb od 0 do n-1:

```rust
fn add_for(n: i32) -> i32 {
    let mut total = 0;
    for i in 0..n {
        total += n;
    }
    total
}
```

```rust
fn add_iter_sum(n: i32) -> i32 {
    (0..n).sum()
}
```

## Iteratory

Powyższa funkcja `add_iter_sum` wykorzystuje fakt, że zakres `(0..n)` jest iteratorem (implementuje trait `Iterator`). Dla wszystkich struktur i enumów będących iteratorami jest dostępne szereg metod agregujących, mapujących i przetwarzających je na najróżniejsze sposoby. Jest to istotne, ponieważ często są one najszybsze jak to tylko możliwe (w miarę możliwości używają w gotowym assemblerze instrukcji SIMD).

Na pewno nie opłaca się pisać takiego kodu...

```rust
/// Ta funkcja dodaje wszystkie elementy wektora do siebie i zwraca wynik
fn c_style_vector_sum(elements: Vec<i32>) -> i32 {
    let mut total = 0;
    for i in 0..elements.len() {
        total += elements[i];
    }
    total
}
```

...z nieco zaskakującej przyczyny - wywołanie `elements[i]` może wywołać tzw. _Panic_.

Rust posiada bardzo niewielki Runtime, który zarządza tzw. Panicami. Panic to błąd, który z założenia jest błędem fatalnym, który wskazuje na to, że coś poszło bardzo źle (na podobę tzw. Kernel Panic w systemach operacyjnych). Podobieństwo tych obu nazw nie jest przypadkowe, ponieważ ten ostatni błąd jest "siatką bezpieczeństwa", która daje gwarancję, że wszystkie elementy zostaną zdealokowane prawidłowo i nie wydarzy się Undefined Behaviour (w postaci segfaulta,, czy to wywołania kodu który usuwa wszystko z dysku, czy przywołania innego demona niewiadomo skąd).

Za każdym razem, gdy dokonujemy indeksowania na wektorze, istnieje możliwość że nastąpi panic. Kompilator zatem sprawdza, czy panic trzeba obsłużyć, czy nie, co zabiera cykle zegara. Narzut ten, mimo że niewielki, to w pętlach które wykonują się dużą ilość razy może okazać się znaczący.

Metody z biblioteki standardowej, np. wspomniany już sum nie sprawdzają, czy i-ty element w wektorze istnieje. Co za tym idzie są szybsze.

Zasada generalna to używać jak najmocniej się da gotowych rozwiązań z języka, ponieważ są to tzw. Zero Cost Abstraction - nigdy nie są wolniejsze, a ich użycie pozwala na zaawansowane wnioskowanie kompilatora o kodzie, co pozwala często na bardziej agresywne optymalizacje.

## Enumy

To jest najbardziej unikatowy element, a jednocześnie fundament języka. W wielu językach enumy występują w podobnej formie:

```rust
enum PlayerStatus {
    Alive,
    Dead
}
```

Jednak w Ruście możliwe jest też zastosowanie enuma podobnie do struktury, ale każdy element może mieć inne składowe, lub w ogóle ich nie mieć:

```rust
enum PlayerStatus {
    Alive { name: String, health: i32 },
    Dead,
}
```

Tym sposobem można sensownie modelować możliwe stany aplikacji. Tak zdefiniowanego enuma możemy konstruować w każdym wariancie, sprawdzać jego stan przy pomocy klauzul `if let`, `match` itd.

```rust
/// Funkcja bierze dane o graczu, i zwraca dane zmodyfikowane
/// (nic się nie dzieje in-place)
fn hit(enemy: &PlayerStatus, force: i32) -> PlayerStatus {
    if let PlayerStatus::Alive { name, health } = enemy {
        let new_health = health - force;
        if new_health > 0 {
            PlayerStatus::Alive {
                name: name.to_owned(),
                health: new_health
            }
        } else { PlayerStatus::Dead }
    } else {
        PlayerStatus::Dead
    }
}
```

Analogiczna funkcja używając `match`:

```rust
fn hit(enemy: &PlayerStatus, force: i32) -> PlayerStatus {
    match enemy {
        // sprawdzamy jednocześnie typ enuma, oraz stawiamy warunek
        PlayerStatus::Alive { name, health } if *health > force { PlayerStatus::Alive { name: name.to_owned(), health: health - force }},
        // w reszcie przypadków potrzebny jest pogrzeb
        _ => PlayerStatus::Dead
    }
}
```

Ten use-case enumów przy modelowaniu stanów jest używany bardzo sowicie w całym języku. Istnieje wbudowany w biblioteke standardową `Option<T>` (T to parametr generyczny, który jest konkretyzozwany przy kompilacji, odpowiednik template w C++).

Uproszczona sygnatura enuma Option:

```rust
pub enum Option<T> {
    Some(T),
    None
}
```

Enum ten służy do obsłużenia sytuacji, gdzie w innych językach wystąpiłby null. Ponieważ każda zmienna musi mieć konkretną wartość, to nie ma null w języku. Jeżeli coś może nie mieć wartości, to opakowywujemy to w Option, co dzieje się też w std:

```rust
fn get_last_element(vector: Vec<i32>) -> Option<&i32> {
    // Jeżeli wektor jest pusty, to ostatniego elementu nie ma.
    let last: Option<&i32> = vector.last();

    last
}

fn get_player_name(p: &PlayerStatus) -> Option<&String> {
    if let PlayerStatus::Alive { name, health } = p {
        Some(name)
    } else {
        None
    }
}
```

Analogiczne jest też działanie typu `Result<T, E>`, którego się używa, gdy errory są rzeczą możliwą, i chcielibyśmy taki error móc obsłużyć.

```rust
// Daj ostatniemu żywemu graczowi w grze pieniądze.
fn award_player_for_win(p: &PlayerStatus) -> Result<i32, String> {
    match p {
        PlayerStatus::Alive { name, health } => Ok(50), // Zwracamy wariant Result::Ok
        PlayerStatus::Dead => Err(
            String::from("Jak to się stało, że martwy wygrał grę?")
        )
    }
}
```
