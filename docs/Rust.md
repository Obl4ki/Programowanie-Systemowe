# Podstawy języka Rust

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

Zmienne domyślnie są immutable. Aby to zmienić należy dodać ```mut```:

```rust
let mut x = 5;
x = 6;
assert!(x == 6);
```

Bez ```mut``` możliwa jest tylko redeklaracja zmiennej:

```rust
let x = 5;
let x = 6;
assert!(x == 6);
```

## Wyrażenia
Sama składni jest interpretowana trochę inaczej niż C. W języku funkcjonuje koncept wyrażeń (expression). Przydaje się to w nienadużywaniu ```mut```. Jest możliwe np. taka technika:

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

W przypadku printowania własnych struktur dana struktura musi mieć zaimplementowany ```trait```. Najprościej mówiąc ```trait``` to nieco rozszerzony mechanizm interfejsów (C#, Java) i klas abstrakcyjnych (C++), który jest jednym z głównych mechanizmów polimorficznych.

Aby móc coś wyprintować w trybie debugowania (flaga :?) potrzebny jest na tym trait ```Debug``` (std::fmt::Debug). Analogicznie sytuacja wygląda z traitem ```Display```.

Debug służy do wyświetlania struktur i enumów na potrzeby programisty, i na tym się skupimy. Aby automatycznie zaimplementować trait debug dla naszej struktury, możemy napisać:

```rust
#[derive(Debug)]
//^^^^^^^^^^^^
struct Point {
    x: f64,
    y: f64
}
```

