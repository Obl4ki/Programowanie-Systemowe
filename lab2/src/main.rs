// żeby skorzystać z libc należy dodać ją jako dependency, a następnie zaimportować w pliku za pomocą instrukcji use
// w terminalu: cargo add libc
use libc::{getpwnam, getutxent, passwd, utmpx, USER_PROCESS};


// uruchomienie programu: cargo run
// opcjonalnie w trybie release: cargo run --release
fn main() {
    // blok unsafe jest potrzebny po to, aby wykonać operację, której wynikiem może być undefined behaviour
    // tutaj jest to dereferencja wskaźnika * (nie referencji - rust poprawność obsługiwania referencji oznaczaną & sprawdza przy kompilacji)
    unsafe {
        // libc w ruście jest analogiczne do libc w c
        let mut entries = getutxent();

        while !entries.is_null() {
            let entry = *entries;

            if entry.ut_type != USER_PROCESS {
                entries = getutxent();
                continue;
            }

            // rzutowanie wskaźnika na typ i8 (*const c_char to to samo co *const i8)
            let userinfo = *getpwnam(entry.ut_user.as_ptr() as *const i8); 

            print_user(entry, userinfo);

            entries = getutxent();
        }
    }
}

// w chwili pisania tej funkcji nie wiedziałem o istnieniu String::from_utf8_lossy
// późniejsze laboratoria korzystają już z funkcji biblioteki standardowej zamiast z takiego fikołka
fn i8_array_to_string(arr: &[i8]) -> String {
    arr.into_iter()
        .map(|char_as_i8| *char_as_i8 as u8 as char)
        .filter(|c| c != &'\0')
        .collect::<String>()
}

fn print_user(u: utmpx, p: passwd) {
    let username = i8_array_to_string(&u.ut_user);
    let uid = p.pw_uid;
    let shell = i8_array_to_string(&u.ut_line);
    let host = i8_array_to_string(&u.ut_host);

    println!("{}\t{}\t{}\t{}", username, uid, host, shell);
}
