use std::path::{Component, Path, PathBuf};

pub fn join(base : &str, paths : Vec<&str>) -> String{
    let mut ret = PathBuf::from(base);
    for v in paths{
        ret.push(v);
    }
    ret.to_str().unwrap().to_string()
}

pub fn clean<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    let mut components = path.components().peekable();
    let mut result = PathBuf::new();

    if let Some(Component::RootDir) = components.peek() {
        result.push(components.next().unwrap());
    }

    while let Some(component) = components.next() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if result.components().count() > 0 {
                    result.pop();
                }
            }
            _ => {
                result.push(component);
            }
        }
    }

    result.to_string_lossy().to_string()
}