// Copyright 2023. The resback authors all rights reserved.

use crate::about;

pub async fn root() -> String {
    about()
}

pub async fn protected() -> &'static str {
    "Hello, World!"
}
