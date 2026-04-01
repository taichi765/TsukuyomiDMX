use std::path::PathBuf;
use tsukuyomi_core::doc::{FixtureDefRegistry, FixtureDefRegistryImpl};

#[test]
fn iter_metadata_works() {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "fixtures"]
        .iter()
        .collect();
    let mut def_rg = FixtureDefRegistryImpl::new(path);

    def_rg.load().expect("should success");

    assert_eq!(2, def_rg.iter_metadata().count());
    assert!(
        def_rg
            .iter_metadata()
            .any(|v| v.manufacturer == "american-dj" && v.model == "mega-tripar-profile-plus")
    );
    assert!(
        def_rg
            .iter_metadata()
            .any(|v| v.manufacturer == "cameo" && v.model == "auro-spot-300")
    );
}
