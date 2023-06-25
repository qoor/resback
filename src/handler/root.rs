// Copyright 2023. The resback authors all rights reserved.

use crate::about;

pub async fn root_handler() -> String {
    about()
}

pub async fn protected_handler() -> &'static str {
    "Hello, World!"
}
