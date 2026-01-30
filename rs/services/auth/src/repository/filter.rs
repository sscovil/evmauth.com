use uuid::Uuid;

use crate::domain::OrgVisibility;

#[derive(Debug, Clone, Default)]
pub struct PersonFilter {
    pub email: Option<String>,
    pub auth_provider: Option<String>,
    pub search: Option<String>,
}

impl PersonFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn auth_provider(mut self, provider: impl Into<String>) -> Self {
        self.auth_provider = Some(provider.into());
        self
    }

    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct OrgFilter {
    pub owner_id: Option<Uuid>,
    pub visibility: Option<OrgVisibility>,
    pub search: Option<String>,
}

impl OrgFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn owner_id(mut self, owner_id: Uuid) -> Self {
        self.owner_id = Some(owner_id);
        self
    }

    pub fn visibility(mut self, visibility: OrgVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }

    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct OrgMemberFilter {
    pub org_id: Option<Uuid>,
    pub member_id: Option<Uuid>,
}

impl OrgMemberFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn org_id(mut self, org_id: Uuid) -> Self {
        self.org_id = Some(org_id);
        self
    }

    pub fn member_id(mut self, member_id: Uuid) -> Self {
        self.member_id = Some(member_id);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntityFilter {
    pub search: Option<String>,
    pub entity_type: Option<String>,
}

impl EntityFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }

    pub fn entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = Some(entity_type.into());
        self
    }
}
