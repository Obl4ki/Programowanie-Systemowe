# Proponowane rozwiązanie do Lab 2

W rozwiązaniu zostanie wykorzystana biblioteka libc.

Pełne rozwiązanie dostępne w katalogu lab2.

## Opis rozwiązania zadania

Korzystając z biblioteki libc, czyli wrappera do biblioteki standardowej z C, możemy uzyskać dostęp do bazy danych utmpx przechowywującej dane o obecnie zalogowanych użytkownikach.

https://man7.org/linux/man-pages/man3/getutxent.3.html
https://docs.rs/libc/latest/libc/fn.getutxent.html

Później mając dane o użytkowniku (konkretnie o jego nazwie), można uzyskać jego dane za pomocą getpwnam.

https://docs.rs/libc/latest/libc/fn.getpwnam.html
https://man7.org/linux/man-pages/man3/getpwnam.3.html

Możemy przez wszystkie zapisy w bazie utmp przejść w następujący sposób:

```rust
    // Uzyskanie następnego zapisu z bazy
    let mut entry_ptr = unsafe { libc::getutxent() };

    // Dopóki nie skończyły się następne zapisy
    while !entry_ptr.is_null() {
        // Dereferencja wskaźnika na zapis (również potrzebne jest unsafe)
        let entry = unsafe { *entry_ptr };

        // To nie jest informacja o użytkowniku, dlatego chcemy zacząć działać na następnym rekordzie
        if entry.ut_type != libc::USER_PROCESS {
            entry_ptr = unsafe { libc::getutxent() };
            continue;
        }

        // Ponieważ rustowa wersja getpwnam zwraca informacje o konkretnym użytkowniku jako rustowy array slice, a chcemy wskaźnik, to musimy dokonać konwersji przy pomocy .as_ptr()
        let userinfo = unsafe { *libc::getpwnam(entry.ut_user.as_ptr()) };

        // Metoda pomocnicza do wyświetlenia danych o tym użytkowniku
        print_user(&entry, &userinfo);

        // Po całym procesie przetwarzamy następny record
        entries = unsafe { libc::getutxent() };
    }
```

Mamy też funkcje pomocnicze do wyświetlania i konwersji z typów zwracanych przez libc na string

```rust
// Podajemy record z utmpx i passwd do funkcji jako referencję
fn print_user(u: &libc::utmpx, p: &libc::passwd) {
    let username = i8_array_to_string(&u.ut_user);
    let uid = p.pw_uid;
    let shell = i8_array_to_string(&u.ut_line);
    let host = i8_array_to_string(&u.ut_host);

    println!("{username}\t{uid}\t{host}\t{shell}");
}

fn i8_array_to_string(arr: &[i8]) -> String {
    // Zmieniamy array slice (referencje na typ i8) na iterator, po czym
    // dla każdego elementu dokonujemy konwersji
    // (na typ u8 istnieje konwersja try_into(), która może się nie powieść)
    // Dla prostoty przykładu odpakowywujemy błąd, zakładając, że
    // rekordy które będziemy podawać są poprawne.

    let u8_arr: Vec<u8> = arr.iter().map(|&x| x.try_into().expect("Wartość powinna zostać skonwertowana poprawnie.")).collect();
    u8_array_to_string(&u8_arr)
}

fn u8_array_to_string(arr: &[u8]) -> String {
    String::from_utf8_lossy(arr).into_owned()
}
```
