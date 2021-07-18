mod tests;

pub type Id = u128;
pub type Token = u128;

#[derive(Default)]
pub struct KVStorage<Id: Default + Eq + std::hash::Hash + Clone, Value: Default + Clone> {
    storage: std::collections::HashMap<Id, Value>,
}

impl<Id: Default + Eq + std::hash::Hash + Clone, Value: Default + Clone> Store<Id, Value>
    for KVStorage<Id, Value>
{
    fn store(&mut self, id: Id, value: Value) {
        self.storage.insert(id, value);
    }

    fn get(&self, id: &Id) -> Option<&Value> {
        self.storage.get(id)
    }

    fn get_mut(&mut self, id: &Id) -> Option<&mut Value> {
        self.storage.get_mut(id)
    }

    fn contains_key(&self, id: &Id) -> bool {
        self.storage.contains_key(id)
    }

    fn list_values(&self) -> Vec<(Id, Value)> {
        self.storage
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

pub trait Store<Id, Value> {
    // We want an auto-incremented ID for most type.
    // TODO: Change that :(
    fn store(&mut self, id: Id, value: Value);
    fn get(&self, id: &Id) -> Option<&Value>;
    fn get_mut(&mut self, id: &Id) -> Option<&mut Value>;
    fn contains_key(&self, id: &Id) -> bool;
    // TODO: This could be an iterator but I don't want to deal with the lifetimes now
    // I'll revisit this once the DB is in use
    // as a first approach I'll deal with pagination later.
    fn list_values(&self) -> Vec<(Id, Value)>;
}

// TODO: Make singleton
#[derive(Default)]
pub struct HRSystem<JobStorage: Store<Id, Job>, CandidateStorage: Store<String, Candidate>> {
    jobs: JobStorage,
    last_id: Id, // <--- TODO: Get rid of this here.
    candidates: CandidateStorage,
}

// TODO: Add permiksioned users(For create_job_posting, and anyhing regarding advancing a process)
impl<JobStorage: Store<Id, Job>, CandidateStorage: Store<String, Candidate>>
    HRSystem<JobStorage, CandidateStorage>
{
    pub fn list_jobs(&self) -> Vec<(Id, Job)> {
        self.jobs.list_values()
    }
    pub fn create_job_posting(&mut self, name: String) -> Id {
        let id = self.last_id;
        self.last_id += 1;
        self.jobs.store(id, Job::new(name));
        id
    }

    pub fn get_job_by_id(&mut self, id: &Id) -> Option<&Job> {
        self.jobs.get(id)
    }

    // TODO: Return error when can't register
    pub fn register_candidate(&mut self, user: String, password: String) -> Result<(), ()> {
        if !self.candidates.contains_key(&user) {
            self.candidates
                .store(user.clone(), Candidate { user, password });
            Ok(())
        } else {
            Err(())
        }
    }

    // TODO: Actually rerturn a token
    pub fn login(&self, user: &String, password: &String) -> Option<Token> {
        if let Some(candidate) = self.candidates.get(user) {
            if &candidate.password == password {
                Some(0)
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

    pub fn apply(&mut self, user: &String, token: Token, job_id: Id) -> Result<(), ()> {
        Self::check_token(user.clone(), token)?;
        // TODO: This would need to hold all candidates in memory
        // change this
        let job = self.jobs.get_mut(&job_id).ok_or(())?;
        if job.state != JobState::Open || job.applicants.contains_key(user) {
            return Err(());
        }
        job.applicants
            .insert(user.clone(), Candidacy::new(user.clone()));
        Ok(())
    }

    // TODO: Lot of duplicated code here
    pub fn interview(&mut self, user: String, job_id: Id) -> Result<(), ()> {
        let job = self.jobs.get_mut(&job_id).ok_or(())?;
        if job.state != JobState::Open {
            return Err(());
        }
        job.applicants
            .entry(user)
            .and_modify(|a| *a = std::mem::take(a).interview()); // <-- That take feels weird but I need a place-holder value to modify a, will take a further look late
        Ok(())
    }

    pub fn approve(&mut self, user: String, job_id: Id) -> Result<(), ()> {
        let job = self.jobs.get_mut(&job_id).ok_or(())?;
        if job.state != JobState::Open {
            return Err(());
        }
        job.applicants
            .entry(user.clone())
            .and_modify(|a| *a = std::mem::take(a).approve()); // <-- That take feels weird but I need a place-holder value to modify a, will take a further look late
        if !matches!(job.applicants.get(&user).ok_or(())?, Candidacy::Approved(_)) {
            return Err(());
        }
        job.state = JobState::Closed;
        Ok(())
    }

    pub fn reject(&mut self, user: String, job_id: Id) -> Result<(), ()> {
        let job = self.jobs.get_mut(&job_id).ok_or(())?;
        if job.state != JobState::Open {
            return Err(());
        }
        job.applicants
            .entry(user)
            .and_modify(|a| *a = std::mem::take(a).reject()); // <-- That take feels weird but I need a place-holder value to modify a, will take a further look late
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct Candidate {
    pub user: String,
    pub password: String,
}

#[derive(PartialEq, Clone)]
pub enum JobState {
    Open,
    Closed, // <-- It'd be good to have a reference here to the application that won, but I won't deal with Pin now
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

// The applicant information in the following types is already conveyed in the applicants
// HashMap, might get rid of it later but won't think too much for now since this will be probably different
// whence I add the DB.
#[derive(Debug, PartialEq, Clone)]
pub struct AppliedApplication {
    applicant: String,
}

impl AppliedApplication {
    fn interview(self) -> InterviewedApplication {
        InterviewedApplication {
            applicant: self.applicant,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct InterviewedApplication {
    applicant: String,
}

impl InterviewedApplication {
    fn approve(self) -> ApprovedApplication {
        ApprovedApplication {
            applicant: self.applicant,
        }
    }

    fn reject(self) -> RejectedApplication {
        RejectedApplication {
            applicant: self.applicant,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RejectedApplication {
    applicant: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ApprovedApplication {
    applicant: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Candidacy {
    Applied(AppliedApplication),
    Interviewed(InterviewedApplication),
    Rejected(RejectedApplication),
    Approved(ApprovedApplication),
}

impl Default for Candidacy {
    fn default() -> Self {
        Self::Applied(AppliedApplication {
            applicant: "".to_string(),
        })
    }
}

impl Candidacy {
    fn new(applicant: String) -> Self {
        Candidacy::Applied(AppliedApplication { applicant })
    }

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
