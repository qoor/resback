// Copyright 2023 The resback authors

use crate::about;

pub async fn root_handler() -> String {
    about()
}
