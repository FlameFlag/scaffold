use std::collections::HashMap;

#[must_use]
pub fn render(input: &str, bindings: &HashMap<&str, &str>) -> String {
    let mut rendered = input.to_owned();
    for (name, value) in bindings {
        rendered = rendered.replace(&format!("{{{{ {name} }}}}"), value);
        rendered = rendered.replace(&format!("{{{{{name}}}}}"), value);
    }
    rendered
}

#[must_use]
pub fn render_slice(input: &[String], bindings: &HashMap<&str, &str>) -> Vec<String> {
    input.iter().map(|item| render(item, bindings)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_spaced_and_compact_placeholders() {
        let bindings = HashMap::from([("tool", "ripgrep"), ("prefix", "/opt/rg")]);

        assert_eq!(
            render("install {{ tool }} into {{prefix}}", &bindings),
            "install ripgrep into /opt/rg"
        );
    }

    #[test]
    fn leaves_unknown_placeholders_intact() {
        let bindings = HashMap::from([("tool", "ripgrep")]);

        assert_eq!(
            render("{{ tool }} {{ missing }}", &bindings),
            "ripgrep {{ missing }}"
        );
    }

    #[test]
    fn renders_each_argv_item_independently() {
        let bindings = HashMap::from([("prefix", "/tmp/demo"), ("tool", "demo")]);
        let rendered = render_slice(
            &[
                "cp".to_owned(),
                "{{tool}}".to_owned(),
                "{{ prefix }}/bin".to_owned(),
            ],
            &bindings,
        );

        assert_eq!(rendered, vec!["cp", "demo", "/tmp/demo/bin"]);
    }
}
