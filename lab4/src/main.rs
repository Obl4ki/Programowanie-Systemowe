// https://docs.rs/nix/latest/nix/
// ważne - niektóre biblioteki korzystają z tzw. features. można dzięki temu wybrać, co potrzebujemy, a reszta feature nie jest analizowana ani dołączana
// https://doc.rust-lang.org/cargo/reference/features.html
// dla tego projektu korzystamy z feature resource
// cargo add nix --feature resource
use nix::sys::resource::{getrusage, UsageWho};
use parse_args::Args;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

mod parse_args;

// ważna sprawa - main zwraca Result<(), String>, gdzie T: (), czyli typ jednostkowy, odpowiednik void, i E, czyli String, wiadomość o błędzie który może się zdarzyć.
// w wielu miejscach maina będziemy chcieli w ten sposób zgłaszać błędy za pomocą operatora ?, i taka deklaracja musi być w tym miejscu
fn main() -> Result<(), String> {
    //cargo run --release -- -t 1 -c "find /etc"
    // wywołanie funkcji pobierającej argumenty - let Typ { verbose, ... } = x od razu rozkłada strukturę x na właściwości składowe
    let Args {
        verbose,
        times,
        command,
    } = parse_args::get();

    // zamiana komendy do wywołania na iterator poprzez rozdzielenie znakami białymi poszczególnych słów
    // https://doc.rust-lang.org/std/iter/index.html
    // za chwilę będziemy przechodzić przez ten iterator, konsumując w nim kolejne elementy instrukcją next. z tego powodu musi być oznaczony jako mut (mutable)
    let mut program_iter = command.split_whitespace().into_iter();

    // next wyciąga następny element z iteratora - elementu po tej operacji w iteratorze już nie ma
    // next zwraca option - jeżeli dostajemy wariant Some(T) - jest jakaś wartość, użytkownik wprowadził komendę w sposób oczekiwany.
    // jeżeli jest None, to metoda .ok_or(...) ustawia wiadomość dla użytkownika o błędzie.
    // ok_or zwraca typ Result<T, E> - tutaj T jest takie jak w Some(T), a E to String który podajemy
    let command = program_iter.next().ok_or(format!(
        "Name of the command to be timed should be specified"
    ))?;
    //^ następnie używamy ? na enumie Result, aby otrzymać wartość T.
    // jeżeli wartość jest wariantu Err, to propagujemy tę wartość w górę. na tym polega mechanizm obsługi błędów

    // zbieramy resztę iteratora w wektor - operator ::<T> w wywołaniu to tzw. turbofish
    // pomaga on się domyślić kompilatorowi, jakim typem ma się posłużyć, gdy funkcja jest generyczna
    // tutaj mówimy, że collect ma zwrócić Vec, przy czym nie potrzebujemy określać co w tym wektorze jest, dlatego że w konsumowanym iteratorze te elementy są i typ danych jest wywnioskowywany.
    let arguments = program_iter.collect::<Vec<_>>();

    let mut command = Command::new(command);
    command.args(arguments);

    let mut total_real = Duration::ZERO;
    let mut total_system = Duration::ZERO;
    let mut total_user = Duration::ZERO;

    for _ in 0..times {
        let start_time = Instant::now();
        let output = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|err| format!("Error spawning the child process: {err:?}"))?;

        let elapsed_time = start_time.elapsed();

        let resource_usage = getrusage(UsageWho::RUSAGE_CHILDREN)
            .map_err(|errno| format!("getrusage failed with code {errno}"))?;

        let system_time = resource_usage.system_time();
        let user_time = resource_usage.user_time();
        let system_dur = Duration::new(
            system_time.tv_sec() as u64,
            system_time.tv_usec() as u32 * 1000,
        );
        let user_dur = Duration::new(user_time.tv_sec() as u64, user_time.tv_usec() as u32 * 1000);

        total_real += elapsed_time;
        total_user += user_dur;
        total_system += system_dur;

        if verbose {
            let process_stdout: String = String::from_utf8_lossy(&output.stdout).into();
            let process_stderr: String = String::from_utf8_lossy(&output.stderr).into();
            println!("{}", process_stdout);
            println!("{}", process_stderr);
        }
    }

    println!("real\t{:.3}s", total_real.as_secs_f64() / times as f64);
    println!("user\t{:.3}s", total_user.as_secs_f64() / times as f64);
    println!("sys\t{:.3}s", total_system.as_secs_f64() / times as f64);

    Ok(())
}
