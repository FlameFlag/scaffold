(library
  (scaffold extensions distro nix profile)
  (export
    nix/profile-add-argv
    nix/profile-install-argv
    nix/profile-remove-argv
    nix/profile-package
    nix/profile-platform)
  (import (rnrs) (scaffold catalog base) (scaffold extensions distro nix base))

  (doc-next
    (signature "(nix/profile-add-argv package-ref ...)")
    (summary "Build argv for `nix profile add`.")
    (param 'package-ref "Installable refs; defaults to `{{ package }}` when omitted."))

  (define nix/profile-add-argv
    (case-lambda
      (() (nix/profile-add-argv "{{ package }}"))
      (package-refs (arr/append-list (nix/argv "profile" "add") package-refs))))

  (doc-next
    (signature "(nix/profile-install-argv package-ref ...)")
    (summary "Build argv for `nix profile install`, an alias of `nix profile add`.")
    (param 'package-ref "Installable refs; defaults to `{{ package }}` when omitted."))

  (define nix/profile-install-argv nix/profile-add-argv)

  (doc-next
    (signature "(nix/profile-remove-argv name ...)")
    (summary "Build argv for `nix profile remove`."))

  (define (nix/profile-remove-argv . names)
    (arr/append-list (nix/argv "profile" "remove") names))

  (doc-next
    (signature "(nix/profile-package name package-ref field ...)")
    (summary "Create a catalog tool installed by `nix profile add`.")
    (param 'name "Catalog tool name.")
    (param 'package-ref "Nix installable such as `nixpkgs#hello`.")
    (param 'field "Additional tool fields that override defaults.")
    (returns "A tool with a package action and Nix profile uninstall metadata."))

  (define (nix/profile-package name package-ref . fields)
    (object/merge
      (tool
        name
        (package (field 'name package-ref) (field 'install-argv (nix/profile-add-argv)))
        (field
          'uninstall
          (uninstall
            (field 'commands (arr (uninstall/command (nix/profile-remove-argv name)))))))
      fields))

  (doc-next
    (signature "(nix/profile-platform predicate package-ref field ...)")
    (summary "Create a package/platform override installed through `nix profile add`.")
    (param 'predicate "Host predicate for this package rule.")
    (param 'package-ref "Nix installable such as `nixpkgs#hello`.")
    (param 'field "Additional platform fields that override defaults."))

  (define (nix/profile-platform predicate-value package-ref . fields)
    (object/merge
      (package/platform
        predicate-value
        (arr "nix")
        (nix/profile-add-argv "{{ package }}")
        (field 'name package-ref))
      fields))

  (moduledoc
    (summary "Nix profile package helpers for catalog tools and platform overrides.")
    (group "Nix profiles")))
