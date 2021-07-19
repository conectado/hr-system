# HR System

This is a toy HR system used completely by CLI.

## Requirements

* [Rust](https://www.rust-lang.org/tools/install)
* [SQLite3](https://www.sqlite.org/index.html) as a dev dependency

## Structure

There is both a lib and bin target included in the project.

### Lib

The lib is composed of a single module in the file `src/lib.rs`.

The most important part of the lib is `System` which is a purely static structure(no state) but internally it access to a lazily initiated (through the use of `lazy_static`) struct that mantains the connection to the DB.

The advantage of this approach against the classic singleton `get_instance` is that you don't have to repeat the `get_instance` throughout all the client's code. Furthermore it makes it rather more obvious that you are accessing a static struct.

### Bin

The binary is the simplest possible CLI, you're presented different main menues depending if you are logged in or not. Navigating the menues is done through the number of the option presented.

It's very important to know to apply to a job you need to be logged in, to advance an interview process or post a new job you must be logged out. (**TODO:** Advancing a process or posting a job is obviously something that shohuld require more permissions than applying, due to time constrains this was not achieved)

### General

There are 3 elements(The DB roughly mimicks these):
* Jobs
* Candidates(or Users)
* Applications

* Jobs: Can be either open or closed, they have a name (**Note:** They don't have a description yet but adding it should be trivial as long as they are not expected to be **too** long)
* Candidates: A candidate is simply someone who can login to the system and apply
* Applications: Is a relationship between a Candidate and a Job it follows one of these processes (And no other): Applied -> Interviewed -> Approved/Rejected. Once an application is approved a Job posting gets closed an no new application can be made and the process can't be advanced.

## TODO
These are things that can be improved:
* The storage backend should be something that one can decide with a feature(Being limited to sqlite isn't nice)
* Security is very bad, passwords aren't salted and tokens are stored in memory(Should be stored in something like keyring) and the hashing algorithms are SHA256 it has to be changed to something like PBKDF2
* Tests should cover more cases, only covers the most common ones 
* `src/lib.rs` has grown too big and should be split
* Using sqlx instead of rusqlite could be an improvement to make the library easily async/await
* Couple of more small TODOs in code
* Improve code documentation
* The CLI is very messy a better interface is in order
* The DB is stored only in `hr_store.db` which could be read from dotenv
