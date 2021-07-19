use hrsystem::{LoggedUser, System};
use lazy_static::lazy_static;
use promptly::prompt;
use std::sync::Mutex;

// This is of course not secure. But I'll not focus on this now.
lazy_static! {
    static ref TOKEN: Mutex<Option<LoggedUser>> = Mutex::new(None);
}

fn main() {
    main_menu();
}

fn main_menu() {
    loop {
        println!("Available Jobs:");
        print_jobs();
        println!("");
        if TOKEN
            .lock()
            .expect("Should be used in single-thread for now")
            .is_none()
        {
            match print_options(&["Add Job", "Register", "Login", "Advance Process"]) {
                0 => job_menu(),
                1 => register(),
                2 => login_menu(),
                3 => advance_process_menu(),
                _ => panic!("Print options should never return an option beyond the option length"),
            }
        } else {
            match print_options(&["Apply", "Logout"]) {
                0 => job_apply(),
                1 => *TOKEN.lock().expect("This is single threaded") = None,
                _ => panic!(
                    "Print options should never return a number beyond the number of options"
                ),
            }
        }
    }
}

fn job_apply() {
    let job_id = prompt("Chose what job to apply").expect("Error reading line");

    let temp_token = TOKEN.lock().expect("Single threaded");
    let logged_user = temp_token
        .as_ref()
        .expect("Should have logged in at this point");
    if System::apply(
        &logged_user.user,
        logged_user.token,
        logged_user.user_id,
        job_id,
    )
    .is_err()
    {
        println!("Already applied or closed\n");
    }
}

fn register() {
    loop {
        let user = prompt("Insert Username").expect("Error reading line");
        // This should use password prompt but promptly doesn't have that.
        let pass = prompt("Insert Password").expect("Error reading line");
        if System::register_candidate(user, pass).is_err() {
            println!("Username already registered")
        } else {
            break;
        }
    }
}

fn job_menu() {
    loop {
        let job_name = prompt("Enter job name").expect("Error reading line");
        if System::create_job_posting(job_name).is_ok() {
            break;
        } else {
            println!("Error creating job posting");
        }
    }
}

fn login_menu() {
    loop {
        let user = prompt("Insert Username").expect("Error reading line");
        // This should use password prompt but promptly doesn't have that.
        let pass = prompt("Insert Password").expect("Error reading line");
        let logged_user = System::login(&user, &pass);
        if let Some(logged_user) = logged_user {
            *TOKEN
                .lock()
                .expect("This should be single-threaded for now") = Some(logged_user);
            break;
        } else {
            println!("Inexistent user or incorrect password")
        }
    }
}

fn advance_process_menu() {
    match print_options(&["Interview", "Approve", "Reject"]) {
        0 => interview(),
        1 => approve(),
        2 => reject(),
        _ => panic!("Print options should never return a number beyond the number of options"),
    }
}

// TODO: Dedup this --
fn interview() {
    let candidate = prompt("Candidate to interview").expect("Read line error");
    let job_id = prompt("Job id of the interview").expect("Read line error");
    if System::interview(candidate, job_id).is_err() {
        println!("There was an error interviewing candidate");
    }
}

fn approve() {
    let candidate = prompt("Candidate to interview").expect("Read line error");
    let job_id = prompt("Job id of the interview").expect("Read line error");
    if System::approve(candidate, job_id).is_err() {
        println!("There was an error approving candidate");
    }
}

fn reject() {
    let candidate = prompt("Candidate to interview").expect("Read line error");
    let job_id = prompt("Job id of the interview").expect("Read line error");
    if System::reject(candidate, job_id).is_err() {
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
fn print_jobs() {
    let jobs = System::list_jobs().expect("DB Connection problems");
    if jobs.is_empty() {
        println!("There are no Jobs posted yet");
    } else {
        jobs.iter().for_each(|job| println!("{}: {}", job.id, job));
    }
}
