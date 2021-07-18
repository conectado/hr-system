#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn create_job_posting() {
        let mut system = HRSystem::<KVStorage<Id, Job>, KVStorage<String, Candidate>>::default();
        let job_posting_id = system.create_job_posting("Engineer".to_string());
        assert_eq!(
            system.get_job_by_id(&job_posting_id).unwrap().name,
            "Engineer".to_string()
        );
    }

    #[test]
    fn register_candidate() {
        let mut system = HRSystem::<KVStorage<Id, Job>, KVStorage<String, Candidate>>::default();
        let _ = system.register_candidate("test".to_string(), "test".to_string());
        // Can login with correct password
        let token = system.login(&"test".to_string(), &"test".to_string());
        assert!(token.is_some());
        // Can't login with incorrect password
        let token = system.login(&"test".to_string(), &"incorrect".to_string());
        assert!(token.is_none());
    }

    #[test]
    fn candidate_process() {
        let mut system = HRSystem::<KVStorage<Id, Job>, KVStorage<String, Candidate>>::default();
        let job_posting_id = system.create_job_posting("Engineer".to_string());
        let _ = system.register_candidate("test".to_string(), "test".to_string());
        let token = system.login(&"test".to_string(), &"test".to_string());
        assert!(system
            .apply("test".to_string(), token.unwrap(), job_posting_id)
            .is_ok());
        assert_eq!(
            system
                .get_job_by_id(&job_posting_id)
                .unwrap()
                .applicants
                .get("test")
                .unwrap(),
            &Candidacy::Applied(AppliedApplication {
                applicant: "test".to_string()
            })
        );

        // Follows flow
        assert!(system.approve("test".to_string(), job_posting_id).is_err());
        assert_eq!(
            system
                .get_job_by_id(&job_posting_id)
                .unwrap()
                .applicants
                .get("test")
                .unwrap(),
            &Candidacy::Applied(AppliedApplication {
                applicant: "test".to_string()
            })
        );

        // Follows flow
        assert!(system.reject("test".to_string(), job_posting_id).is_ok());
        assert_eq!(
            system
                .get_job_by_id(&job_posting_id)
                .unwrap()
                .applicants
                .get("test")
                .unwrap(),
            &Candidacy::Applied(AppliedApplication {
                applicant: "test".to_string()
            })
        );

        assert!(system.interview("test".to_string(), job_posting_id).is_ok());
        assert_eq!(
            system
                .get_job_by_id(&job_posting_id)
                .unwrap()
                .applicants
                .get("test")
                .unwrap(),
            &Candidacy::Interviewed(InterviewedApplication {
                applicant: "test".to_string()
            })
        );

        assert!(system.approve("test".to_string(), job_posting_id).is_ok());
        assert_eq!(
            system
                .get_job_by_id(&job_posting_id)
                .unwrap()
                .applicants
                .get("test")
                .unwrap(),
            &Candidacy::Approved(ApprovedApplication {
                applicant: "test".to_string()
            })
        );

        let _ = system.register_candidate("test1".to_string(), "test".to_string());
        let token = system.login(&"test1".to_string(), &"test".to_string());
        println!("{:?}", token);
        assert!(system
            .apply("test1".to_string(), token.unwrap(), job_posting_id)
            .is_err());
    }
}
