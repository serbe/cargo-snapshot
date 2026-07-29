use crate::model::project_kind::ProjectKind;

pub(crate) struct Metadata<'a> {
    pub kind: ProjectKind<'a>,
    pub dependencies: Vec<String>,
}
