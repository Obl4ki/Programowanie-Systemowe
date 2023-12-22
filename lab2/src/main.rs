// uruchomienie programu: cargo run
// opcjonalnie w trybie release: cargo run --release
fn main() {
    // blok unsafe jest potrzebny po to, aby wykonać operację, której wynikiem może być undefined behaviour
    // tutaj jest to dereferencja wskaźnika * (nie referencji - rust poprawność obsługiwania referencji oznaczaną & sprawdza przy kompilacji)
        let mut entries = unsafe { libc::getutxent() };

        while !entries.is_null() {
            let entry = unsafe { *entries };

            if entry.ut_type != libc::USER_PROCESS {
                entries = unsafe { libc::getutxent() };
                continue;
            }

            let userinfo = unsafe { *libc::getpwnam(entry.ut_user.as_ptr()) }; 

            print_user(&entry, &userinfo);

            entries = unsafe { libc::getutxent() };
        }
    
}

fn i8_array_to_string(arr: &[i8]) -> String {
    let u8_arr: Vec<u8> = arr.iter().map(|&x| x.try_into().expect("Wartość powinna zostać skonwertowana poprawnie.")).collect();
    u8_array_to_string(&u8_arr)
}

fn u8_array_to_string(arr: &[u8]) -> String {
    String::from_utf8_lossy(arr).into_owned()
}

fn print_user(u: &libc::utmpx, p: &libc::passwd) {
    let username = i8_array_to_string(&u.ut_user);
    let uid = p.pw_uid;
    let shell = i8_array_to_string(&u.ut_line);
    let host = i8_array_to_string(&u.ut_host);

    println!("{username}\t{uid}\t{host}\t{shell}");
}
