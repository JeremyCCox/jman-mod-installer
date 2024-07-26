use std::ffi::OsString;
use std::fs;
use std::fs::ReadDir;
use std::path::PathBuf;
use ssh2::File;

pub fn list_profiles_mods(profile_path:&str) -> Vec<PathBuf> {
    let mods = fs::read_dir([profile_path, "mods"].join("/")).expect("Could not read dir!");
    let mut mod_names = Vec::new();
    for x in mods {
        let entry = x.unwrap();
        let val = entry.path();
        mod_names.push(val);
    };
    mod_names
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn list_mods(){
        let mods = list_profiles_mods(&["fake.minecraft","profiles","test_profile"].join("/"));
        assert_eq!(mods,[PathBuf::from("../fake.minecraft/profiles/test_profile/mods\\testjar.jar")])
    }
}