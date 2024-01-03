# Proponowane rozwiązanie do Lab 3

W rozwiązaniu zostanią wykorzystane biblioteki:

- libc - integracji z systemem linux,
- clap - do łatwego obsłużenia flag w programie
- itertools - rozszerza mechanizm iteratorów o nowe metody, przydatne głównie dla wygody
- libloading - biblioteka pozwalająca na załadowanie biblioteki dynamicznej .so lub .dll

Pełne rozwiązanie jest dostępne w katalogu lab3.

Pierwszą rzeczą, którą należy zrobić, jest zmiana trybu kompilacji biblioteki (lib.rs) do biblioteki dynamicznej.
Można to zrobić w Cargo.toml projektu

```toml
[lib]
crate-type = ["dylib"]
```

Następnie dodajemy biblioteki poniżej w Cargo.toml:

```toml
[dependencies]
clap = { version = "4.1.11", features = ["cargo"] }
itertools = "0.12.0"
libc = "0.2.151"
libloading = "0.8.1"
```

Można ewentualnie dokonać tego z konsoli:

```bash
cargo add clap --features cargo
cargo add itertools libc libloading
```

Po tej konfiguracji tworzymy plik lib.rs, który będzie zawierał kod dla naszej biblioteki dynamicznej.
Logika aplikacji jest podobna do poprzedniego laboratorium, natomiast doszły nowe elementy, takie jak przełączniki
-g (groups) i -h (hosts)

Fragment programu:

```rust
pub fn print_all_users(h: bool, g: bool) {
        let mut entries = unsafe { getutxent() };

        while !entries.is_null() {
            let entry = unsafe { *entries };

            if entry.ut_type != USER_PROCESS {
                entries = unsafe { getutxent() };
                continue;
            }

            let userinfo = unsafe { *getpwnam(entry.ut_user.as_ptr() as *const i8) };

            let username = get_username(entry);
            let groups = unsafe { get_groups(&entry, &userinfo) };
            let hosts = get_hosts(&entry);

            // Poniżej doklejamy napisy z informacjami, jeżeli odpowiadające przełączniki są ustawione na true
            let console_out = username;

            let console_out = if h {
                format!("{console_out} {hosts}")
            } else {
                console_out
            };

            let console_out = if g {
                format!("{console_out} {groups}")
            } else {
                console_out
            };

            // Wyświetlenie wyniku
            println!("{console_out}");
            entries = unsafe { getutxent() };
        }

}
```

Do kodu dodajemy funkcję, która zwraca grupy
(jest to kod bardzo podobny do kodu analogicznego w C):

```rust
unsafe fn get_groups(u: &utmpx, p: &passwd) -> String {
        let mut n = 0;

        let n_groups: *mut i32 = &mut n; // getgrouplist

        // getgrouplist to funkcja która pozwala otrzymać dane o grupach
        // https://man7.org/linux/man-pages/man3/getgrouplist.3.html
        // https://rust-lang.github.io/hashbrown/libc/fn.getgrouplist.html
        getgrouplist(
            u.ut_user.as_ptr() as *const c_char, // dokonujemy potrzebnych konwersji
            p.pw_gid,
            std::ptr::null_mut(),   // tak się deklaruje pusty wskaźnik (mut dlatego, że w deklaracji
                                    // getgrouplist jest wymagany groups: *mut gid_t)
            n_groups,
        );

        // tak, za pomocą libc można zrobić malloc
        let groups = malloc(size_of::<gid_t>() * n as usize) as *mut gid_t;

        getgrouplist(
            u.ut_user.as_ptr() as *const c_char,
            p.pw_gid,
            groups, // do tego wskaźnika mają być zapisane grupy
            n_groups,
        );

        //
        let group_names = (0..n) // dla n grup
            .map(|gid_idx| { // całość funkcji map zmienia jeden typ/rodzaj danych w kolekcji w drugi
                             // w tym przypadku będzie to zmiana numera grupy na jej nazwę

                let gid_id = *groups.offset(gid_idx as isize); // przesuwamy wskaźnik na start grupy do i-tej grupy
                let groups = *getgrgid(gid_id); // otrzymujemy grupę o określonym id

                groups.gr_name // zwracamy nazwę grupy
            })
            .map(|str_ptr| ptr_to_string(str_ptr).expect("Group names should all be ok")) // konwertujemy i zakładamy że operacja przebiegła pomyślnie, wyciągając wartość
            .collect::<Vec<String>>(); // konsumujemy iterator, wraz z deklaracją że chcemy otrzymać wektor stringów

        format!("[{}]", group_names.join(", "))

}

// funkcja pomocnicza konwertująca wskaźnik na wartość reprezentującą albo string, albo nic, jeżeli operacja się nie powiedzie.
// typ Option jest odpowiednikiem null w innych językach
// żeby dokonać dereferencji wskaźnika oznaczamy całą funkcję jako unsafe - dlatego kod wywołujący tę funkcję też będzie musiał być unsafe
unsafe fn ptr_to_string(ptr: *mut i8) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let c_str = CStr::from_ptr(ptr);

    match c_str.to_str() {
        Ok(s) => Some(s.to_owned()),
        Err(_) => None,
    }
}
```

Jest to wszystko, co potrzebne aby zadziałała biblioteka dynamiczna.

Zawartość pliku main.rs jest dużo prostsza i zawiera importy bibliotek, zadeklarowanie potrzebnych flag, import całej funkcjonalności `print_all_users` z biblioteki dynamicznej oraz wywołanie jej.

```rust
use clap::{command, Arg, ArgAction};

#[allow(special_module_name)]

fn main() {
    // Za pomocą tego kodu tworzymy
    let matches = command!()
        .disable_help_flag(true) // wyłączamy flagę help - nie będzie nam ona potrzebna
        .arg(
            Arg::new("hosts") // nazwa flagi
                .short('h') // krótki przełącznik
                .long("hosts") // długi przełącznik
                .action(ArgAction::SetTrue), // jeżeli przełącznik będzie użyty, to bool ma być true
        )
        .arg(
            Arg::new("groups")
                .short('g')
                .long("groups")
                .action(ArgAction::SetTrue),
        )
        .get_matches(); // ostateczne wczytanie flag

    // wyciąganie booleanów do zmiennych
    let h = matches.get_flag("hosts");
    let g = matches.get_flag("groups");

    // załadowanie biblioteki dynamicznej
    // poprawność operacji jest niemożliwa do zweryfikowania przez język, więc trzeba otoczyć kod blokiem unsafe
    unsafe {
        let lib = libloading::Library::new("liblab3.so").unwrap(); // pod taką nazwą tworzy się biblioiteka dynamiczna w folderze target po uruchomieniu

        // zadeklarowanie za pomocą typu (po dwukropku) jakich parametrów funkcji się spodziewamy, i przekazanie jej nazwy poprzez parametr
        // w typie <unsafe extern fn(bool, bool)> poszczególne terminy oznaczają
        // extern - funkcja pochodząci z innej jednostki tłumaczenia - translation unit - tutaj biblioteka dynamiczna
        // unsafe - funkcja, której bezpieczeństwo nie jest dowiedzione statycznie i trzeba wziąć pod uwagę, że może wywołać błąd (np. segfault lub inny)
        // fn(bool, bool) - wskaźnik na funkcję przyjmujący 2 booleany i nic nie zwracający
        // gdyby funkcja miała coś zwracać, to wyglądałoby to np. tak:
        // fn(bool, bool) -> String
        // Uwaga: podana sygnatura funkcji nie jest wskazówką, a zabezpieczeniem, i musi być poprawna, aby kod zadziałał. Jeżeli nawet znajdzie się w bibliotece funkcja o takiej samej nazwie, ale innej sygnaturze, to biblioteka jej nie wczyta.
        let print_all_users: libloading::Symbol<unsafe extern fn(bool, bool)> = lib.get(b"print_all_users").unwrap(); // odpakowywujemy błąd, ponieważ wiemy że ta biblioteka jest poprawna i że istnieje

        print_all_users(h,  g);
    }
}
```

Program możemy wywołać z argumentami w sposób tradycyjny albo w sposób zintegrowany z cargo run, przekazując argumenty do programu:

```bash
cargo run -- -h -g
```
