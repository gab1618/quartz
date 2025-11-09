use std::path::PathBuf;

use openapi::Spec;
use quartz_core::Quartz;

pub fn init_quartz_from_openapi_spec(path: PathBuf, spec: Spec) {
    let initialized = Quartz::init(path, PathBuf::default()).unwrap();
    for (path, operations) in spec.paths {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_quartz() {
        let examples_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
        println!("{}", examples_path.display());
        let a = openapi::from_path(examples_path.join("simple-api-overview.json")).unwrap();
    }
}
