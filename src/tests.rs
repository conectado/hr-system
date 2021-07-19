#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn create_job_posting() {
        let mut system = HRSystem::new();
        let job_posting_id = system.create_job_posting("Engineer".to_string()).unwrap();
        assert_eq!(
            system.get_job_by_id(&job_posting_id).unwrap().name,
            "Engineer".to_string()
        );
    }

    #[test]
    fn register_candidate() {
        let mut system = HRSystem::new();
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
        let mut system = HRSystem::new();
        let job_posting_id = system.create_job_posting("Engineer".to_string()).unwrap();
        let _ = system.register_candidate("test".to_string(), "test".to_string());
        let _ = system.register_candidate("test1".to_string(), "test".to_string());
        let logged_in_user = system
            .login(&"test1".to_string(), &"test".to_string())
            .unwrap();
        assert!(system
            .apply(
                &logged_in_user.user,
                logged_in_user.token,
                logged_in_user.user_id,
                job_posting_id
            )
            .is_ok());
        assert_eq!(
            system
                .list_jobs()
                .unwrap()
                .first()
                .unwrap()
                .applicants
                .get("test1")
                .unwrap(),
            &Candidacy::Applied(AppliedApplication)
        );

        // Must follow flow
        assert!(system.approve("test1".to_string(), job_posting_id).is_err());
        assert_eq!(
            system
                .list_jobs()
                .unwrap()
                .first()
                .unwrap()
                .applicants
                .get("test1")
                .unwrap(),
            &Candidacy::Applied(AppliedApplication)
        );

        // Must follow flow
        assert!(system.reject("test1".to_string(), job_posting_id).is_ok());
        assert_eq!(
            system
                .list_jobs()
                .unwrap()
                .first()
                .unwrap()
                .applicants
                .get("test1")
                .unwrap(),
            &Candidacy::Applied(AppliedApplication)
        );
        // Reject Flow
        assert!(system
            .interview("test1".to_string(), job_posting_id)
            .is_ok());
        assert_eq!(
            system
                .list_jobs()
                .unwrap()
                .first()
                .unwrap()
                .applicants
                .get("test1")
                .unwrap(),
            &Candidacy::Interviewed(InterviewedApplication)
        );

        assert!(system.reject("test1".to_string(), job_posting_id).is_ok());
        assert_eq!(
            system
                .list_jobs()
                .unwrap()
                .first()
                .unwrap()
                .applicants
                .get("test1")
                .unwrap(),
            &Candidacy::Rejected(RejectedApplication)
        );

        // Aprove Flow

        let logged_in_user = system
            .login(&"test".to_string(), &"test".to_string())
            .unwrap();
        assert!(system
            .apply(
                &logged_in_user.user,
                logged_in_user.token,
                logged_in_user.user_id,
                job_posting_id
            )
            .is_ok());
        assert_eq!(
            system
                .list_jobs()
                .unwrap()
                .first()
                .unwrap()
                .applicants
                .get("test")
                .unwrap(),
            &Candidacy::Applied(AppliedApplication)
        );
        assert!(system.interview("test".to_string(), job_posting_id).is_ok());
        assert_eq!(
            system
                .list_jobs()
                .unwrap()
                .first()
                .unwrap()
                .applicants
                .get("test")
                .unwrap(),
            &Candidacy::Interviewed(InterviewedApplication)
        );

        assert!(system.approve("test".to_string(), job_posting_id).is_ok());
        assert_eq!(
            system
                .list_jobs()
                .unwrap()
                .first()
                .unwrap()
                .applicants
                .get("test")
                .unwrap(),
            &Candidacy::Approved(ApprovedApplication)
        );

        // Gets closed
        let _ = system.register_candidate("test2".to_string(), "test".to_string());
        let logged_in_user = system
            .login(&"test2".to_string(), &"test".to_string())
            .unwrap();
        assert!(system
            .apply(
                &"test2".to_string(),
                logged_in_user.token,
                logged_in_user.user_id,
                job_posting_id
            )
            .is_err());
    }
}
