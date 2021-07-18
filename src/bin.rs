use hrsystem::{Candidate, HRSystem, Id, Job, KVStorage, Token};
use lazy_static::lazy_static;
use promptly::prompt;
use std::sync::Mutex;

struct LoggedUser {
    user: String,
    token: Token,
}

// This is of course not secure. But I'll not focus on this now.
lazy_static! {
    static ref TOKEN: Mutex<Option<LoggedUser>> = Mutex::new(None);
}

fn main() {
    let mut system = HRSystem::<KVStorage<Id, Job>, KVStorage<String, Candidate>>::default();
    main_menu(&mut system);
}

fn main_menu(system: &mut HRSystem<KVStorage<Id, Job>, KVStorage<String, Candidate>>) {
    loop {
        println!("Available Jobs:");
        print_jobs(&system);
        println!("");
        if TOKEN
            .lock()
            .expect("Should be used in single-thread for now")
            .is_none()
        {
            match print_options(&["Add Job", "Register", "Login", "Advance Process"]) {
                0 => job_menu(system),
                1 => register(system),
                2 => login_menu(system),
                3 => advance_process_menu(system),
                _ => panic!("Print options should never return an option beyond the option length"),
            }
        } else {
            match print_options(&["Apply", "Logout"]) {
                0 => job_apply(system),
                1 => *TOKEN.lock().expect("This is single threaded") = None,
                _ => panic!(
                    "Print options should never return a number beyond the number of options"
                ),
            }
        }
    }
}

fn job_apply(system: &mut HRSystem<KVStorage<Id, Job>, KVStorage<String, Candidate>>) {
    let job_id = prompt("Chose what job to apply").expect("Error reading line");

    let temp_token = TOKEN.lock().expect("Single threaded");
    let logged_user = temp_token
        .as_ref()
        .expect("Should have logged in at this point");
    if system
        .apply(&logged_user.user, logged_user.token, job_id)
        .is_err()
    {
        println!("Already applied or closed");
    }
}

fn register(system: &mut HRSystem<KVStorage<Id, Job>, KVStorage<String, Candidate>>) {
    loop {
        let user = prompt("Insert Username").expect("Error reading line");
        // This should use password prompt but promptly doesn't have that.
        let pass = prompt("Insert Password").expect("Error reading line");
        if system.register_candidate(user, pass).is_err() {
            println!("Inexistent user or incorrect password")
        } else {
            break;
        }
    }
}

fn job_menu(system: &mut HRSystem<KVStorage<Id, Job>, KVStorage<String, Candidate>>) {
    let job_name = prompt("Enter job name").expect("Error reading line");
    system.create_job_posting(job_name);
}

fn login_menu(system: &mut HRSystem<KVStorage<Id, Job>, KVStorage<String, Candidate>>) {
    let mut token = None;
    while token.is_none() {
        let user = prompt("Insert Username").expect("Error reading line");
        // This should use password prompt but promptly doesn't have that.
        let pass = prompt("Insert Password").expect("Error reading line");
        token = system.login(&user, &pass);
        if let Some(token) = token {
            *TOKEN
                .lock()
                .expect("This should be single-threaded for now") =
                Some(LoggedUser { user, token });
        } else {
            println!("Inexistent user or incorrect password")
        }
    }
}

fn advance_process_menu(system: &mut HRSystem<KVStorage<Id, Job>, KVStorage<String, Candidate>>) {
    match print_options(&["Interview", "Approve", "Reject"]) {
        0 => interview(system),
        1 => approve(system),
        2 => reject(system),
        _ => panic!("Print options should never return a number beyond the number of options"),
    }
}

// TODO: Dedup this --
fn interview(system: &mut HRSystem<KVStorage<Id, Job>, KVStorage<String, Candidate>>) {
    let candidate = prompt("Candidate to interview").expect("Read line error");
    let job_id = prompt("Job id of the interview").expect("Read line error");
    if system.interview(candidate, job_id).is_err() {
        println!("There was an error interviewing candidate");
    }
}

fn approve(system: &mut HRSystem<KVStorage<Id, Job>, KVStorage<String, Candidate>>) {
    let candidate = prompt("Candidate to interview").expect("Read line error");
    let job_id = prompt("Job id of the interview").expect("Read line error");
    if system.approve(candidate, job_id).is_err() {
        println!("There was an error approving candidate");
    }
}

fn reject(system: &mut HRSystem<KVStorage<Id, Job>, KVStorage<String, Candidate>>) {
    let candidate = prompt("Candidate to interview").expect("Read line error");
    let job_id = prompt("Job id of the interview").expect("Read line error");
    if system.reject(candidate, job_id).is_err() {
        println!("There was an error rejecting candidate");
    }
}
// --

// TODO: This could dispatch the corresponding menu
fn print_options(options: &[&str]) -> usize {
    options
        .iter()
        .enumerate()
        .for_each(|(i, v)| println!("{}. {}", i, v));

    loop {
        let res = prompt("Choose option").expect("Error reading line");
        if res < options.len() {
            return res;
        } else {
            println!("Invalid option");
        }
    }
}
// TODO: Make this generic on storage
fn print_jobs(system: &HRSystem<KVStorage<Id, Job>, KVStorage<String, Candidate>>) {
    let jobs = system.list_jobs();
    if jobs.is_empty() {
        println!("There are no Jobs posted yet");
    } else {
        system
            .list_jobs()
            .iter()
            .for_each(|(job_id, job)| println!("{}: {}", job_id, job));
    }
}
