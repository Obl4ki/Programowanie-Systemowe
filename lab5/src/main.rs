use chrono::Timelike;
use nix::sys::signal::{
    sigaction, sigprocmask, SaFlags, SigAction, SigHandler, SigmaskHow, Signal,
};
use nix::sys::signalfd::SigSet;
use rand::{Rng, SeedableRng};

use nix::libc::{self, exit, siginfo_t};

use nix::unistd;

use anyhow::Result;

use std::ffi::{c_int, c_void};

mod parse_args;
use parse_args::Args;

fn random_in_range(min: i32, max: i32) -> i32 {
    let pid = nix::unistd::getpid().as_raw();
    let mut rng = rand::rngs::StdRng::seed_from_u64(pid as u64);
    rng.gen_range(min..max + 1)
}

extern "C" fn print_child_end_action(_sig_no: c_int, sig_info: *mut siginfo_t, _xd: *mut c_void) {
    let now = chrono::Local::now();
    let mins = now.minute();
    let secs = now.second();

    let pid = unsafe { (*sig_info).si_pid() };
    let status = unsafe { (*sig_info).si_status() };
    println!("czas ukończenia: {mins}:{secs} | pid: {pid} | status: {status}");
}

extern "C" fn await_children_handler(_: libc::c_int) {
    println!("\nEnding - awaiting rest of child processes");

    while unsafe { CHILD_COUNT > 0 } {
        unsafe { libc::sleep(5) };
    }

    unsafe { exit(0) };
}

extern "C" fn set_spawn_child_flag_handler(_: libc::c_int) {
    unsafe {
        SPAWN_CHILD_FLAG = 1;
    }
}

extern "C" fn loop_break_handler(_: libc::c_int) {
    unsafe { exit(TTL) };
}

// zmienne globalne, które będą należeć do procesów potomnych
// żeby móc odczytać lub zapisać, trzeba odczyt lub zapis otoczyć blokiem unsafe {}
// ponieważ zmienne globalne są niebezpieczne i mogą prowadzić do undefined behavior
static mut CHILD_COUNT: i32 = 0;
static mut SPAWN_CHILD_FLAG: i32 = 1;
static mut TTL: i32 = 0;

fn main() -> Result<()> {
    let _num_children = 0;
    let _args: Vec<String> = std::env::args().collect();

    let Args {
        w: _thread_create_interval,
        m: _thread_ttl,
    } = parse_args::get();

    let sig_child_action = SigAction::new(
        SigHandler::SigAction(print_child_end_action),
        SaFlags::SA_SIGINFO,
        SigSet::all(),
    );
    unsafe { sigaction(Signal::SIGCHLD, &sig_child_action)? };

    let sig_alarm_handler = SigAction::new(
        SigHandler::Handler(set_spawn_child_flag_handler),
        SaFlags::all(),
        SigSet::all(),
    );
    unsafe { sigaction(Signal::SIGALRM, &sig_alarm_handler)? };

    let sig_int_handler = SigAction::new(
        SigHandler::Handler(await_children_handler),
        SaFlags::all(),
        SigSet::all(),
    );
    unsafe { sigaction(Signal::SIGINT, &sig_int_handler)? };

    let child_sigalarm_handler = SigAction::new(
        SigHandler::Handler(loop_break_handler),
        SaFlags::all(),
        SigSet::all(),
    );
    loop {
        if unsafe { SPAWN_CHILD_FLAG } == 1 {
            unsafe { SPAWN_CHILD_FLAG = 0 };
            unistd::alarm::set(_thread_create_interval as u32);

            unsafe { CHILD_COUNT += 1 };

            match unsafe { unistd::fork() }? {
                unistd::ForkResult::Parent { child: _ } => {}
                unistd::ForkResult::Child => {
                    let mut rng = rand::thread_rng();
                    let secs: u32 = rng.gen_range(2..10);
                    unistd::alarm::set(secs);

                    let mut block_sigint = nix::sys::signal::SigSet::empty();
                    block_sigint.add(Signal::SIGINT);
                    sigprocmask(SigmaskHow::SIG_BLOCK, Some(&block_sigint), None)?;

                    unsafe { TTL = random_in_range(1, _thread_ttl as i32) };

                    let mut cur_time = libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    };

                    unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut cur_time) };

                    let time_info = unsafe { libc::localtime(&cur_time.tv_sec) };
                    let mins = (unsafe { *time_info }).tm_min;
                    let secs = (unsafe { *time_info }).tm_sec;

                    println!(
                        "Child PID {} TTL {} {}:{}",
                        unistd::getpid(),
                        unsafe { TTL },
                        mins,
                        secs
                    );

                    unsafe { sigaction(Signal::SIGALRM, &child_sigalarm_handler) }?;

                    unistd::alarm::set(unsafe { TTL } as u32);

                    let mut endless_task_buf = 0;
                    loop {
                        endless_task_buf = (endless_task_buf + 1) % 5;
                    }
                }
            }
        } else {
            unistd::sleep(5);
        }
    }
}
