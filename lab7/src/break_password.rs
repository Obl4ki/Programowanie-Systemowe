use crate::gen_hash::{self, EtcShadowEntry};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use scoped_threadpool::Pool;
use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub fn break_password(
    shadow_entry: EtcShadowEntry,
    pass_dict_path: impl AsRef<Path>,
) -> Result<Option<String>> {
    let mut word_file = File::open(pass_dict_path)?;
    let mut word_reader = String::new();
    word_file.read_to_string(&mut word_reader)?;

    // ograniczyłem listę haseł do 80000 funkcją take
    let password_list: Vec<&str> = word_reader.lines().take(80000).collect();

    // dodanie paska postępu z biblioteki indicatif
    let progress_bar = ProgressBar::new(password_list.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );

    progress_bar.reset();
    let sequential_res = time_method(|| {
        find_sequentially(password_list.clone(), shadow_entry.clone(), &progress_bar)
    });

    println!("Dla wyszukiwania sekwencyjnego: {sequential_res:?}");

    progress_bar.reset();
    let rayon_res = time_method(|| {
        find_using_rayon(password_list.clone(), shadow_entry.clone(), &progress_bar)
    });

    println!("Dla biblioteki rayon: {rayon_res:?}");

    progress_bar.reset();
    let threadpool_res = time_method(|| {
        find_using_threadpool(password_list.clone(), shadow_entry.clone(), &progress_bar)
    });

    println!("Dla thread poola: {threadpool_res:?}");

    Ok(threadpool_res.0)
}

/// Funkcja pomocnicza przyjmująca funkcję, mierząca jej czas wykonania i zwracająca krotkę
/// zawierającą jej wynik oraz czas wykonania
fn time_method<F, R>(method: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let res = method();
    let elapsed = Instant::now() - start;
    (res, elapsed)
}

fn find_sequentially(
    password_list: Vec<&str>,
    shadow_entry: EtcShadowEntry,
    progress_bar: &ProgressBar,
) -> Option<String> {
    let found = password_list
        .into_iter()
        .find(|password| {
            progress_bar.inc(1);
            shadow_entry.repr() == gen_hash::get_entry(password, &shadow_entry.salt).repr()
        })
        .map(String::from);
    progress_bar.finish();

    found
}

// biblioteka rayon służy do tego, aby zamienić iterator w parallel iterator. ten typ danych obsługuje metody działające współbieżnie na iteratorze
// biblioteka automatycznie na podstawie rozmiaru iteratora oraz ilości rdzeni dobiera optymalną ilość threadów
fn find_using_rayon(
    password_list: Vec<&str>,
    shadow_entry: EtcShadowEntry,
    progress_bar: &ProgressBar,
) -> Option<String> {
    let found = password_list
        .into_par_iter()
        .find_any(|password| {
            progress_bar.inc(1);
            shadow_entry.repr() == gen_hash::get_entry(password, &shadow_entry.salt).repr()
        })
        .map(String::from);
    progress_bar.finish();

    found
}

fn find_using_threadpool(
    password_list: Vec<&str>,
    shadow_entry: EtcShadowEntry,
    progress_bar: &ProgressBar,
) -> Option<String> {
    // Arc - Atomic Reference Counter jest po to, żeby można było wysłać posiadaną referencję na inny thread
    // Arc<Mutex<T>> umożliwia wysłanie mutexa zawierającego dowolną wartość na thread, a następnie w threadzie uzyskanie do niej wyłącznego dostępu na potrzebny czas
    let found = Arc::new(Mutex::new(Option::None));
    let mut pool = Pool::new(num_cpus::get() as u32);

    pool.scoped(|scoped| {
        for password in &password_list {
            let found_clone = found.clone();
            let shadow_entry_clone = shadow_entry.clone();
            let progress_bar_clone = progress_bar.clone();

            if found_clone.lock().unwrap().is_some() {
                progress_bar.finish();
                return;
            }

            scoped.execute(move || {
                if found_clone.lock().unwrap().is_some() {
                    return;
                }

                if shadow_entry_clone.repr()
                    == gen_hash::get_entry(password, &shadow_entry_clone.salt).repr()
                {
                    let mut found_guard = found_clone.lock().unwrap(); // uzyskanie dostępu do wartości w mutexie i następnie przypisanie
                    *found_guard = Some(password.to_string()); // gdy mutex lock przestaje istnieć, to mutex zdejmuje blokadę
                }

                progress_bar_clone.inc(1);
            });
        }
    });

    progress_bar.finish();

    // Rozebranie Arc<Mutex<T>> na wartość T i zwrócenie jej
    Arc::into_inner(found).unwrap().into_inner().unwrap()
}
