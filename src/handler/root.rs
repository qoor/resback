use crate::about;

pub async fn root_handler() -> String {
    about()
}
