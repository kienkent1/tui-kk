pub struct ContainerListPage {
    page: i128,
    limit: i32,
}

impl ContainerListPage {
    pub fn new() -> Self {
        Self {
            page: 1,
            limit: 100,
        }
    }
}
