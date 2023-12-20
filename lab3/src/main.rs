use clap::{command, Arg, ArgAction};

#[allow(special_module_name)]

fn main() {
    // cargo add clap
    let matches = command!()
        .disable_help_flag(true)
        .arg(
            Arg::new("hosts")
                .short('h')
                .long("hosts")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("groups")
                .short('g')
                .long("groups")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let h = matches.get_flag("hosts");
    let g = matches.get_flag("groups");
        //cargo run - bez argumentow
        // cargo run -- -h -g

    // utworzenie biblioteki:
    // cargo add libloading
    // https://docs.rs/libloading/latest/libloading/
    unsafe {
        let lib = libloading::Library::new("liblab3.so").unwrap();
        let print_all_users: libloading::Symbol<unsafe extern fn(bool, bool)> = lib.get(b"print_all_users").unwrap();
        print_all_users(h,  g);
    }
}
