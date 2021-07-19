mod tests;

pub type Id = i64;
pub type Token = u128;

// TODO: Make singleton
pub struct HRSystem {
    store: DBStore,
}

// TODO: Add permisioned users(For create_job_posting, and anyhing regarding advancing a process)
// TODO: Error types
impl HRSystem {
    pub fn new() -> Self {
        Self {
            store: DBStore::new(),
        }
    }

    pub fn list_jobs(&self) -> Result<Vec<Job>> {
        self.store.list_jobs()
    }

    pub fn create_job_posting(&mut self, name: String) -> Result<Id> {
        self.store.add_job_posting(&Job::new(name))
    }

    pub fn get_job_by_id(&mut self, id: &Id) -> Option<Job> {
        self.store.get_job_by_id(*id).ok()
    }

    pub fn register_candidate(&mut self, user: String, password: String) -> Result<(), ()> {
        self.store
            .add_candidate(&Candidate {
                id: Default::default(),
                user,
                password,
            })
            .map(|_| ())
            .map_err(|_| ())
    }

    // TODO: Actually rerturn a token
    pub fn login(&self, user: &String, password: &String) -> Option<LoggedUser> {
        if let Ok(candidate) = self.store.get_candidate(user) {
            if &candidate.password == password {
                Some(LoggedUser {
                    user_id: candidate.id,
                    user: candidate.user,
                    token: 0,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn check_token(_user: String, token: Token) -> Result<(), ()> {
        // TODO: OFC, change this
        if token == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn apply(
        &mut self,
        user: &String,
        token: Token,
        candidate_id: Id,
        job_id: Id,
    ) -> Result<Id, ()> {
        Self::check_token(user.clone(), token)?;
        // TODO: This would need to hold all candidates in memory
        // change this
        let job = self.store.get_job_by_id(job_id).map_err(|_| ())?;

        if job.state != JobState::Open {
            Err(())
        } else {
            self.store
                .insert_application(&Application {
                    job_id: job.id,
                    candidate_id,
                    state: Candidacy::default(),
                })
                .map_err(|_| ())
        }
    }

    fn advance_process<F>(&mut self, user: String, job_id: Id, advance: F) -> Result<(), ()>
    where
        F: FnOnce(Candidacy) -> Candidacy,
    {
        let job = self.store.get_job_by_id(job_id).map_err(|_| ())?;
        if job.state != JobState::Open {
            return Err(());
        } else {
            let candidate = self.store.get_candidate(&user).map_err(|_| ())?;
            let mut application = self
                .store
                .get_application(job_id, candidate.id)
                .map_err(|_| ())?;
            // TODO: Here if it doesn't change we could return an err that would save some operations
            application.state = advance(application.state);
            self.store
                .update_application(&application)
                .map(|_| ())
                .map_err(|_| ())
        }
    }

    pub fn interview(&mut self, user: String, job_id: Id) -> Result<(), ()> {
        self.advance_process(user, job_id, |s| s.interview())
    }

    pub fn approve(&mut self, user: String, job_id: Id) -> Result<(), ()> {
        let candidate = self.store.get_candidate(&user).map_err(|_| ())?;
        if self.advance_process(user, job_id, |s| s.approve()).is_ok()
            && matches!(
                self.store
                    .get_application(job_id, candidate.id)
                    .map_err(|_| ())?
                    .state,
                Candidacy::Approved(_)
            )
        {
            let mut job = self.store.get_job_by_id(job_id).map_err(|_| ())?;
            job.state = JobState::Closed;
            self.store.update_job_posting(&job).map_err(|_| ())?;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn reject(&mut self, user: String, job_id: Id) -> Result<(), ()> {
        self.advance_process(user, job_id, |s| s.reject())
    }
}

pub struct LoggedUser {
    pub user: String,
    pub user_id: Id,
    pub token: Token,
}

#[derive(Default, Clone)]
pub struct Candidate {
    pub id: Id,
    pub user: String,
    pub password: String,
}

#[derive(PartialEq, Clone, Copy)]
pub enum JobState {
    Open = 0,
    Closed = 1, // <-- It'd be good to have a reference here to the application that won, but I won't deal with Pin now
}

impl From<u8> for JobState {
    fn from(i: u8) -> Self {
        match i {
            0 => Self::Open,
            1 => Self::Closed,
            _ => Self::Open,
        }
    }
}

impl Default for JobState {
    fn default() -> Self {
        Self::Open
    }
}

impl std::fmt::Display for JobState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "{}", "Open"),
            Self::Closed => write!(f, "{}", "Close"),
        }
    }
}

#[derive(Default, Clone)]
pub struct Job {
    pub id: Id,
    pub name: String,
    pub applicants: std::collections::HashMap<String, Candidacy>,
    pub state: JobState,
}

impl Job {
    fn new(name: String) -> Self {
        Job {
            name,
            ..Job::default()
        }
    }
}
impl std::fmt::Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, State: {}\n Applicants: {:?}",
            self.name, self.state, self.applicants
        )
    }
}

struct Application {
    pub job_id: Id,
    pub candidate_id: Id,
    pub state: Candidacy,
}

// The applicant information in the following types is already conveyed in the applicants
// HashMap, might get rid of it later but won't think too much for now since this will be probably different
// whence I add the DB.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AppliedApplication;

impl AppliedApplication {
    fn interview(self) -> InterviewedApplication {
        InterviewedApplication
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct InterviewedApplication;

impl InterviewedApplication {
    fn approve(self) -> ApprovedApplication {
        ApprovedApplication
    }

    fn reject(self) -> RejectedApplication {
        RejectedApplication
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RejectedApplication;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ApprovedApplication;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Candidacy {
    Applied(AppliedApplication),
    Interviewed(InterviewedApplication),
    Rejected(RejectedApplication),
    Approved(ApprovedApplication),
}

impl From<u8> for Candidacy {
    fn from(i: u8) -> Self {
        match i {
            0 => Self::Applied(AppliedApplication),
            1 => Self::Interviewed(InterviewedApplication),
            2 => Self::Rejected(RejectedApplication),
            3 => Self::Approved(ApprovedApplication),
            _ => Default::default(),
        }
    }
}

impl From<Candidacy> for u8 {
    fn from(candidacy: Candidacy) -> Self {
        match candidacy {
            Candidacy::Applied(_) => 0,
            Candidacy::Interviewed(_) => 1,
            Candidacy::Rejected(_) => 2,
            Candidacy::Approved(_) => 3,
        }
    }
}

impl Default for Candidacy {
    fn default() -> Self {
        Self::Applied(AppliedApplication)
    }
}

impl Candidacy {
    // Might use Result<Self, Self> or something like that later
    fn interview(self) -> Self {
        if let Self::Applied(application) = self {
            Self::Interviewed(application.interview())
        } else {
            self
        }
    }

    // Might use Result<Self, Self> or something like that later
    fn approve(self) -> Self {
        if let Self::Interviewed(application) = self {
            Self::Approved(application.approve())
        } else {
            self
        }
    }

    fn reject(self) -> Self {
        if let Self::Interviewed(application) = self {
            Self::Rejected(application.reject())
        } else {
            self
        }
    }
}

use rusqlite::{params, Connection, Result};
struct DBStore {
    conn: Connection,
}

// TODO: Methods here can be part of a TRAIT to be implemented in any Backend(diesel, in-memory, sqlx, etc..)
// TODO: Also! A lot of these methods return vec or create vecs calling the DB, this is not ideal, we should've some kind of pagination
impl DBStore {
    fn new() -> Self {
        DBStore {
            conn: Self::setup_db(),
        }
    }

    fn setup_db() -> Connection {
        // TODO: dotenv to configure this
        // TODO quickest way to do testing, but should have some kind of configuration
        #[cfg(not(test))]
        let conn = Connection::open("hr_store.db").unwrap();

        #[cfg(test)]
        let conn = Connection::open_in_memory().unwrap();

        conn.execute(
            "
            create table if not exists jobs (
                id integer primary key,
                name text not null unique,
                state integer not null
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "
            create table if not exists candidates (
                id integer primary key,
                name text unique,
                password text not null
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "
            create table if not exists applications (
                state integer not null,
                job_id int,
                candidate_id int,
                FOREIGN KEY (job_id) REFERENCES jobs(id),
                FOREIGN KEY (candidate_id) REFERENCES candidates(id)
                PRIMARY KEY (job_id, candidate_id)
            )",
            [],
        )
        .unwrap();

        conn
    }

    fn get_application(&self, job_id: Id, candidate_id: Id) -> Result<Application> {
        self.conn.query_row(
            "SELECT state  FROM applications WHERE candidate_id = (?1) AND job_id = (?2)",
            [candidate_id, job_id],
            |row| {
                Ok(Application {
                    job_id,
                    candidate_id,
                    state: row.get::<_, u8>(0)?.into(),
                })
            },
        )
    }
    fn add_job_posting(&self, job: &Job) -> Result<Id> {
        let state = job.state as u8;
        self.conn.execute(
            "INSERT INTO jobs (name, state) values (?1, ?2)",
            params![job.name, state],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    fn list_jobs(&self) -> Result<Vec<Job>> {
        let mut stmt = self.conn.prepare(
            "SELECT jobs.id, jobs.name, jobs.state, applications.state, candidates.name
            FROM jobs
            LEFT JOIN applications ON applications.job_id = jobs.id
            LEFT JOIN candidates ON candidates.id = applications.candidate_id",
        )?;

        let mut rows = stmt.query([])?;

        let mut job_map = std::collections::HashMap::<Id, Job>::new();
        while let Some(row) = rows.next()? {
            let mut default_dict: std::collections::HashMap<String, Candidacy> = Default::default();
            if row.get::<_, u8>(3).is_ok() && row.get::<_, String>(4).is_ok() {
                default_dict.insert(row.get(4).unwrap(), row.get::<_, u8>(3).unwrap().into());
            }
            job_map
                .entry(row.get(0)?)
                .and_modify(|e| {
                    if row.get::<_, u8>(3).is_ok() && row.get::<_, String>(4).is_ok() {
                        e.applicants
                            .insert(row.get(4).unwrap(), row.get::<_, u8>(3).unwrap().into());
                    }
                })
                .or_insert(Job {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    state: row.get::<_, u8>(2)?.into(),
                    applicants: default_dict,
                });
        }

        Ok(job_map.into_iter().map(|(_, job)| job).collect())
    }

    // Note: This doesn't actually construct a Job because it doesn't contain the applicants
    // This, is more efficient since I never need the applicant when getting a job by ID.
    fn get_job_by_id(&self, job_id: Id) -> Result<Job> {
        self.conn.query_row(
            "SELECT name, state FROM jobs where id = (?1)",
            [job_id],
            |row| {
                Ok(Job {
                    id: job_id,
                    name: row.get(0)?,
                    state: row.get::<_, u8>(1)?.into(),
                    applicants: Default::default(),
                })
            },
        )
    }

    fn add_candidate(&self, candidate: &Candidate) -> Result<usize> {
        self.conn.execute(
            "INSERT INTO candidates (name, password) values (?1, ?2)",
            [&candidate.user, &candidate.password],
        )
    }

    fn get_candidate(&self, candidate_name: &str) -> Result<Candidate> {
        self.conn.query_row(
            "SELECT id, name, password FROM candidates WHERE name = (?1)",
            &[candidate_name],
            |row| {
                Ok(Candidate {
                    id: row.get(0)?,
                    user: row.get(1)?,
                    password: row.get(2)?,
                })
            },
        )
    }

    fn update_job_posting(&self, job: &Job) -> Result<usize> {
        let state = job.state as u8;
        self.conn.execute(
            "UPDATE jobs SET name = (?1), state = (?2) where id = (?3)",
            params![job.name, state, job.id],
        )
    }

    fn insert_application(&self, application: &Application) -> Result<Id> {
        let state: u8 = application.state.into();
        self.conn.execute(
            "INSERT INTO applications (job_id, candidate_id, state) values (?1, ?2, ?3)",
            params![&application.job_id, &application.candidate_id, state],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    fn update_application(&self, application: &Application) -> Result<usize> {
        let state: u8 = application.state.into();
        self.conn.execute(
            "UPDATE  applications SET state = (?3) WHERE job_id = (?1) AND candidate_id = (?2)",
            params![&application.job_id, &application.candidate_id, state,],
        )
    }
}
