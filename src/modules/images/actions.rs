use bollard::models::{ImageInspect, ImageSummary};
use serde::{Deserialize, Serialize};

use crate::modules::images::image_dto::RunImageRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageAction {
    Refresh,
    Loaded(u64, Result<Vec<ImageSummary>, String>),
    OpenDetail(String),
    DetailLoaded(String, Result<ImageInspect, String>),
    OpenRun {
        image_id: String,
        image_ref: String,
    },
    RunImageInspectLoaded(String, Result<ImageInspect, String>),
    RunContainer(RunImageRequest),
    RunCompleted(Result<String, String>),
    Remove(String, bool),
    Prune,
    ActionCompleted(Result<String, String>),
    BackToList,

    SearchPush(char),
    SearchPop,
    SearchClear,
    CycleSort,
    ToggleOrder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageButtonAction {
    Run,
    Remove,
}
