use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct FileFilter {
    pub org_id: Option<Uuid>,
    pub uploader_id: Option<Uuid>,
    pub mime_type: Option<String>,
    pub search: Option<String>,
}

impl FileFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn org_id(mut self, org_id: Uuid) -> Self {
        self.org_id = Some(org_id);
        self
    }

    pub fn uploader_id(mut self, uploader_id: Uuid) -> Self {
        self.uploader_id = Some(uploader_id);
        self
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct DocFilter {
    pub org_id: Option<Uuid>,
    pub uploader_id: Option<Uuid>,
    pub mime_type: Option<String>,
    pub search: Option<String>,
}

impl DocFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn org_id(mut self, org_id: Uuid) -> Self {
        self.org_id = Some(org_id);
        self
    }

    pub fn uploader_id(mut self, uploader_id: Uuid) -> Self {
        self.uploader_id = Some(uploader_id);
        self
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ImageFilter {
    pub org_id: Option<Uuid>,
    pub uploader_id: Option<Uuid>,
    pub mime_type: Option<String>,
    pub search: Option<String>,
    pub min_width: Option<i32>,
    pub min_height: Option<i32>,
}

impl ImageFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn org_id(mut self, org_id: Uuid) -> Self {
        self.org_id = Some(org_id);
        self
    }

    pub fn uploader_id(mut self, uploader_id: Uuid) -> Self {
        self.uploader_id = Some(uploader_id);
        self
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }

    pub fn min_width(mut self, width: i32) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn min_height(mut self, height: i32) -> Self {
        self.min_height = Some(height);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct MediaFilter {
    pub org_id: Option<Uuid>,
    pub uploader_id: Option<Uuid>,
    pub mime_type: Option<String>,
    pub search: Option<String>,
    pub min_duration_ms: Option<i32>,
    pub max_duration_ms: Option<i32>,
}

impl MediaFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn org_id(mut self, org_id: Uuid) -> Self {
        self.org_id = Some(org_id);
        self
    }

    pub fn uploader_id(mut self, uploader_id: Uuid) -> Self {
        self.uploader_id = Some(uploader_id);
        self
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }

    pub fn min_duration_ms(mut self, duration: i32) -> Self {
        self.min_duration_ms = Some(duration);
        self
    }

    pub fn max_duration_ms(mut self, duration: i32) -> Self {
        self.max_duration_ms = Some(duration);
        self
    }
}
