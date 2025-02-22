pub(crate) fn validate_path_component(component: impl AsRef<str>) -> Result<(), String> {
    let component_str = component.as_ref();

    for c in component_str.chars() {
        if c.is_control() {
            return Err("Path component cannot contain control characters".to_owned());
        }

        if c == '/' || c == '\\' || c == ':' {
            return Err("Path component cannot contain reserved characters".to_owned());
        }
    }

    Ok(())
}
