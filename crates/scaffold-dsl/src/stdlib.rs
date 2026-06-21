use std::borrow::Cow;

use scheme_rs::{
    env::{AllowList, ImportPolicy},
    symbols::Symbol,
};

use super::DocumentationSource;
pub(super) struct Stdlib {
    pub(super) path: &'static str,
    pub(super) name: &'static [&'static str],
    pub(super) source: &'static str,
    pub(super) visibility: StdlibVisibility,
}

pub struct CapabilityDescriptor {
    pub library_name: &'static [&'static str],
    pub library: &'static str,
    pub bridge_library_name: &'static [&'static str],
    pub bridge_library: &'static str,
    pub effect: &'static str,
    pub modes: &'static [CapabilityMode],
    pub docs_source: &'static str,
    pub notes: &'static str,
}

pub struct CapabilityMode {
    pub name: &'static str,
    pub availability: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StdlibVisibility {
    Public,
    Internal,
}

pub(super) const SCAFFOLD_CORE_STDLIB: &[Stdlib] = &[
    Stdlib {
        path: "src/dsl/std/core/vector.scm",
        name: &["scaffold", "core", "vector"],
        source: include_str!("std/core/vector.scm"),
        visibility: StdlibVisibility::Internal,
    },
    Stdlib {
        path: "src/dsl/std/core/object.scm",
        name: &["scaffold", "core", "object"],
        source: include_str!("std/core/object.scm"),
        visibility: StdlibVisibility::Internal,
    },
    Stdlib {
        path: "src/dsl/std/core/doc.scm",
        name: &["scaffold", "core", "doc"],
        source: include_str!("std/core/doc.scm"),
        visibility: StdlibVisibility::Internal,
    },
    Stdlib {
        path: "src/dsl/std/config/vector.scm",
        name: &["scaffold", "config", "vector"],
        source: include_str!("std/config/vector.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/config/object.scm",
        name: &["scaffold", "config", "object"],
        source: include_str!("std/config/object.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/config/documentation.scm",
        name: &["scaffold", "config", "documentation"],
        source: include_str!("std/config/documentation.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/config.scm",
        name: &["scaffold", "config"],
        source: include_str!("std/config.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/path.scm",
        name: &["scaffold", "path"],
        source: include_str!("std/path.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/fs.scm",
        name: &["scaffold", "fs"],
        source: include_str!("std/fs.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/root.scm",
        name: &["scaffold", "catalog", "root"],
        source: include_str!("std/catalog/root.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/action.scm",
        name: &["scaffold", "catalog", "action"],
        source: include_str!("std/catalog/action.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/dependency.scm",
        name: &["scaffold", "catalog", "dependency"],
        source: include_str!("std/catalog/dependency.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/uninstall.scm",
        name: &["scaffold", "catalog", "uninstall"],
        source: include_str!("std/catalog/uninstall.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/tool.scm",
        name: &["scaffold", "catalog", "tool"],
        source: include_str!("std/catalog/tool.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/archive.scm",
        name: &["scaffold", "catalog", "archive"],
        source: include_str!("std/catalog/archive.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/metadata.scm",
        name: &["scaffold", "catalog", "metadata"],
        source: include_str!("std/catalog/metadata.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/platform.scm",
        name: &["scaffold", "catalog", "platform"],
        source: include_str!("std/catalog/platform.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/check.scm",
        name: &["scaffold", "catalog", "check"],
        source: include_str!("std/catalog/check.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/transform.scm",
        name: &["scaffold", "catalog", "transform"],
        source: include_str!("std/catalog/transform.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/helper.scm",
        name: &["scaffold", "catalog", "helper"],
        source: include_str!("std/catalog/helper.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog/base.scm",
        name: &["scaffold", "catalog", "base"],
        source: include_str!("std/catalog/base.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/catalog.scm",
        name: &["scaffold", "catalog"],
        source: include_str!("std/catalog.scm"),
        visibility: StdlibVisibility::Public,
    },
    Stdlib {
        path: "src/dsl/std/test.scm",
        name: &["scaffold", "test"],
        source: include_str!("std/test.scm"),
        visibility: StdlibVisibility::Public,
    },
];

const RUST_BACKED_CAPABILITIES: &[CapabilityDescriptor] = &[
    CapabilityDescriptor {
        library_name: &["scaffold", "path"],
        library: "(scaffold path)",
        bridge_library_name: &["scaffold", "path", "builtins"],
        bridge_library: "(scaffold path builtins)",
        effect: "pure",
        modes: &[
            CapabilityMode {
                name: "catalog",
                availability: "available",
            },
            CapabilityMode {
                name: "test",
                availability: "available",
            },
            CapabilityMode {
                name: "editor",
                availability: "reference-only",
            },
            CapabilityMode {
                name: "wasm",
                availability: "reference-only",
            },
        ],
        docs_source: "src/dsl/std/path.scm",
        notes: "Lexical path operations only; path values do not need to exist.",
    },
    CapabilityDescriptor {
        library_name: &["scaffold", "host"],
        library: "(scaffold host)",
        bridge_library_name: &["scaffold", "host", "builtins"],
        bridge_library: "(scaffold host builtins)",
        effect: "host-read-only",
        modes: &[
            CapabilityMode {
                name: "catalog",
                availability: "available",
            },
            CapabilityMode {
                name: "test",
                availability: "available",
            },
            CapabilityMode {
                name: "editor",
                availability: "reference-only",
            },
            CapabilityMode {
                name: "wasm",
                availability: "reference-only",
            },
        ],
        docs_source: "src/dsl/std/host.scm",
        notes: "Reads host facts, environment variables, and PATH resolution without shelling out.",
    },
    CapabilityDescriptor {
        library_name: &["scaffold", "workspace"],
        library: "(scaffold workspace)",
        bridge_library_name: &["scaffold", "workspace", "generated"],
        bridge_library: "(scaffold workspace generated)",
        effect: "context-read-only",
        modes: &[
            CapabilityMode {
                name: "catalog",
                availability: "available",
            },
            CapabilityMode {
                name: "test",
                availability: "available",
            },
            CapabilityMode {
                name: "editor",
                availability: "reference-only",
            },
            CapabilityMode {
                name: "wasm",
                availability: "reference-only",
            },
        ],
        docs_source: "src/dsl/std/workspace.scm",
        notes: "Exposes paths supplied by DslEvalContext; it does not inspect the filesystem.",
    },
    CapabilityDescriptor {
        library_name: &["scaffold", "fs"],
        library: "(scaffold fs)",
        bridge_library_name: &["scaffold", "fs", "builtins"],
        bridge_library: "(scaffold fs builtins)",
        effect: "host-read-only",
        modes: &[
            CapabilityMode {
                name: "catalog",
                availability: "available",
            },
            CapabilityMode {
                name: "test",
                availability: "available",
            },
            CapabilityMode {
                name: "editor",
                availability: "reference-only",
            },
            CapabilityMode {
                name: "wasm",
                availability: "reference-only",
            },
        ],
        docs_source: "src/dsl/std/fs.scm",
        notes: "Read-only existence/type predicates for absolute paths; no listing, reading, or mutation.",
    },
];

pub(crate) const fn rust_backed_capabilities() -> &'static [CapabilityDescriptor] {
    RUST_BACKED_CAPABILITIES
}

pub(super) fn core_documentation_sources() -> Vec<DocumentationSource> {
    SCAFFOLD_CORE_STDLIB
        .iter()
        .map(|library| DocumentationSource {
            path: library.path,
            source: library.source,
        })
        .chain([
            DocumentationSource {
                path: "src/dsl/std/host.scm",
                source: include_str!("std/host.scm"),
            },
            DocumentationSource {
                path: "src/dsl/std/workspace.scm",
                source: include_str!("std/workspace.scm"),
            },
        ])
        .collect()
}

pub(super) fn import_policy(
    bundled_libraries: &[SchemeLibrary],
    user_libraries: &[SchemeLibrary],
) -> ImportPolicy {
    let mut allow_list = AllowList::from_slice(&[&["rnrs"]]);
    for library in SCAFFOLD_CORE_STDLIB {
        if library.visibility == StdlibVisibility::Public {
            add_static_library(&mut allow_list, library.name);
        }
    }
    for library in bundled_libraries {
        add_owned_library(&mut allow_list, &library.name);
    }
    for library in user_libraries {
        add_owned_library(&mut allow_list, &library.name);
    }
    for capability in rust_backed_capabilities() {
        add_static_library(&mut allow_list, capability.library_name);
    }
    ImportPolicy::allow_only(allow_list)
}

pub(super) fn define_core_libraries(
    runtime: &scheme_rs::runtime::Runtime,
) -> Result<(), scheme_rs::exceptions::Exception> {
    for library in SCAFFOLD_CORE_STDLIB
        .iter()
        .filter(|library| !library_needs_context(library))
    {
        runtime.def_lib(library.source)?;
    }
    Ok(())
}

pub(super) fn define_context_libraries(
    runtime: &scheme_rs::runtime::Runtime,
) -> Result<(), scheme_rs::exceptions::Exception> {
    for library in SCAFFOLD_CORE_STDLIB
        .iter()
        .filter(|library| library_needs_context(library))
    {
        runtime.def_lib(library.source)?;
    }
    Ok(())
}

pub(super) fn define_scheme_libraries(
    runtime: &scheme_rs::runtime::Runtime,
    libraries: &[SchemeLibrary],
) -> Result<(), scheme_rs::exceptions::Exception> {
    let mut pending = libraries.iter().collect::<Vec<_>>();

    while !pending.is_empty() {
        let starting_len = pending.len();
        let mut next = Vec::new();

        for library in pending {
            if runtime.def_lib(&library.source).is_err() {
                next.push(library);
            }
        }

        if next.len() == starting_len {
            return runtime.def_lib(&next[0].source);
        }
        pending = next;
    }

    Ok(())
}

#[derive(Debug)]
pub(super) struct SchemeLibrary {
    pub(super) name: Vec<String>,
    pub(super) source: Cow<'static, str>,
}

fn add_static_library(allow_list: &mut AllowList, name: &'static [&'static str]) {
    allow_list.add_lib(
        name.iter()
            .map(|component| Symbol::intern(component))
            .collect(),
    );
}

fn add_owned_library(allow_list: &mut AllowList, name: &[String]) {
    allow_list.add_lib(
        name.iter()
            .map(|component| Symbol::intern(component))
            .collect(),
    );
}

fn library_needs_context(library: &Stdlib) -> bool {
    matches!(
        library.name,
        ["scaffold", "catalog", "helper" | "base"] | ["scaffold", "catalog"]
    )
}
