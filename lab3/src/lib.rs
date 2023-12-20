use libc::{
    c_char, getgrgid, getgrouplist, getpwnam, getutxent, gid_t, malloc, passwd, utmpx, USER_PROCESS,
};
use std::ffi::CStr;
use std::mem::size_of;

#[no_mangle]
pub extern "C" fn print_all_users(h: bool, g: bool) {
    unsafe {
        let mut entries = getutxent();

        while !entries.is_null() {
            let entry = *entries;

            if entry.ut_type != USER_PROCESS {
                entries = getutxent();
                continue;
            }

            let userinfo = *getpwnam(entry.ut_user.as_ptr() as *const i8);

            let username = get_username(entry);
            let groups = get_groups(&entry, &userinfo);
            let hosts = get_hosts(&entry);

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

            println!("{console_out}");
            entries = getutxent();
        }
    }
}

fn i8_array_to_string(arr: &[i8]) -> String {
    arr.into_iter()
        .map(|char_as_i8| *char_as_i8 as u8 as char)
        .filter(|c| c != &'\0')
        .collect::<String>()
}

fn get_username(u: utmpx) -> String {
    let username = i8_array_to_string(&u.ut_user);

    return format!("{}\t", username);
}
fn get_hosts(u: &utmpx) -> String {
    format!("({})", i8_array_to_string(&u.ut_host))
}

fn get_groups(u: &utmpx, p: &passwd) -> String {
    unsafe {
        let mut n = 0;

        let n_groups: *mut i32 = &mut n;
        getgrouplist(
            u.ut_user.as_ptr() as *const c_char,
            p.pw_gid,
            std::ptr::null_mut(),
            n_groups,
        );

        let groups = malloc(size_of::<gid_t>() * n as usize) as *mut gid_t;

        getgrouplist(
            u.ut_user.as_ptr() as *const c_char,
            p.pw_gid,
            groups,
            n_groups,
        );

        let group_names = (0..n)
            .map(|gid_idx| {
                let gid_id = *groups.offset(gid_idx as isize);
                let groups = *getgrgid(gid_id);

                groups.gr_name
            })
            .map(|str_ptr| ptr_to_string(str_ptr).expect("Group names should all be ok"))
            .collect::<Vec<String>>();

        format!("[{}]", group_names.join(", "))
    }
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
