(import
  (rnrs)
  (scaffold catalog)
  (scaffold test)
  (scaffold extensions target distrobox))

(define direct
  (tool
    "direct"
    (distrobox/package
      "fedora"
      (package (field 'install-argv (arr "dnf" "install" "{{ package }}"))))))

(define platform
  (tool
    "platform"
    (distrobox/package
      "fedora"
      (package
        (field
          'platforms
          (arr
            (package/platform 'linux (arr "dnf") (arr "dnf" "install" "{{ package }}"))))))))

(define platform-multi
  (tool
    "platform-multi"
    (distrobox/package
      "fedora"
      (package
        (field
          'platforms
          (arr
            (package/platform-argvs
              'linux
              (arr "dnf")
              (arr (arr "dnf" "makecache") (arr "dnf" "install" "{{ package }}")))))))))

(define checked
  (distrobox/tool
    "fedora"
    (tool
      "checked"
      (package (field 'install-argv (arr "dnf" "install" "{{ package }}")))
      (field 'checks (arr (check (arr "rpm" "-q" "{{ package }}"))))
      (field
        'uninstall
        (uninstall
          (field
            'commands
            (arr (uninstall/command (arr "dnf" "remove" "{{ package }}")))))))))

(assert/equal
  (arr "distrobox" "enter" "fedora" "--" "dnf" "install" "{{ package }}")
  (object/ref (object/ref direct 'action) 'install-argv))

(assert/equal
  (arr "distrobox" "enter" "fedora" "--" "dnf" "install" "{{ package }}")
  (object/ref
    (vector-ref (object/ref (object/ref platform 'action) 'platforms) 0)
    'install-argv))

(assert/equal
  (arr "distrobox" "enter" "fedora" "--" "dnf" "makecache")
  (vector-ref
    (object/ref
      (vector-ref (object/ref (object/ref platform-multi 'action) 'platforms) 0)
      'install-argvs)
    0))

(assert/equal
  (arr "distrobox" "enter" "fedora" "--" "dnf" "install" "{{ package }}")
  (vector-ref
    (object/ref
      (vector-ref (object/ref (object/ref platform-multi 'action) 'platforms) 0)
      'install-argvs)
    1))

(assert/equal
  (arr "distrobox" "enter" "fedora" "--" "rpm" "-q" "{{ package }}")
  (object/ref (vector-ref (object/ref checked 'checks) 0) 'argv))

(assert/equal
  (arr "distrobox" "enter" "fedora" "--" "dnf" "remove" "{{ package }}")
  (object/ref
    (vector-ref (object/ref (object/ref checked 'uninstall) 'commands) 0)
    'argv))

(moduledoc
  (summary
    "Fixture for Distrobox transformations across direct, platform, and checked tools."))

(extern-doc direct
  (signature "(direct ...)")
  (summary "Direct package action fixture wrapped by Distrobox."))

(extern-doc platform
  (signature "(platform ...)")
  (summary "Platform package action fixture wrapped by Distrobox."))

(extern-doc platform-multi
  (signature "(platform-multi ...)")
  (summary "Platform package action fixture with multiple install commands."))

(extern-doc checked
  (signature "(checked ...)")
  (summary
    "Tool fixture whose install, check, and uninstall argv are wrapped by Distrobox."))
