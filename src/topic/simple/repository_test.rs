mod tests {
    use crate::topic::simple::*;
    use crate::topic::*;
    use std::path::{Path, PathBuf};

    const TEST_DATA_DIR: &'static str = "./test-data";

    fn test_directory() -> &'static Path {
        Path::new(TEST_DATA_DIR) //.join(Path::new(&(name.to_owned() + ".log")))
    }
}
