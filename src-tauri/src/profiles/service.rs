use crate::{
    domain::ProfileId,
    persistence::{RepoError, Repository},
};

pub struct ProfileService<'a> {
    repository: &'a mut Repository,
}

impl<'a> ProfileService<'a> {
    pub fn new(repository: &'a mut Repository) -> Self {
        Self { repository }
    }

    pub fn activate(&mut self, profile_id: &ProfileId) -> Result<(), RepoError> {
        self.repository.activate_profile(profile_id)
    }

    pub fn active(&self) -> Result<Option<ProfileId>, RepoError> {
        self.repository.active_profile()
    }

    pub fn list(&self) -> Result<Vec<ProfileId>, RepoError> {
        self.repository.profiles()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::ProfileId, persistence::Repository};

    #[test]
    fn active_profile_is_tracker_owned_and_profile_list_is_stable() {
        let mut repository = Repository::in_memory().unwrap();
        let first = ProfileId::new("1849811906").unwrap();
        let second = ProfileId::new("249165455").unwrap();
        let mut profiles = ProfileService::new(&mut repository);

        profiles.activate(&second).unwrap();
        profiles.activate(&first).unwrap();

        assert_eq!(profiles.active().unwrap(), Some(first.clone()));
        assert_eq!(profiles.list().unwrap(), vec![first, second]);
    }
}
