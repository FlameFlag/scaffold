(library
  (scaffold host)
  (export
    host/os
    host/arch
    host/platform
    command/available
    command/available?
    command/path
    command/path?
    host/matches?
    env/var
    env/var?)
  (import (rnrs) (scaffold config) (scaffold host builtins))

  (extern-doc host/os
    (signature "host/os")
    (summary "Symbol for the current operating system.")
    (returns
      "A host OS symbol such as `linux`, populated by Scaffold before evaluation."))

  (define host/os '@HOST_OS@)

  (extern-doc host/arch
    (signature "host/arch")
    (summary "Symbol for the current CPU architecture."))

  (define host/arch '@HOST_ARCH@)

  (extern-doc host/platform
    (signature "host/platform")
    (summary "String describing the detected host platform."))

  (define host/platform @HOST_PLATFORM@)

  (extern-doc command/available
    (signature "command/available")
    (summary "Sorted vector of executable command names detected on the host `PATH`.")
    (returns "A vector of command names available to Scaffold subprocesses.")
    (hidden))

  (define command/available (%command/available))

  (extern-doc command/available?
    (signature "(command/available? name)")
    (summary "Return whether a command is available on the current host.")
    (param 'name "Command name to check.")
    (returns "`#t` when the command is detected, otherwise `#f`."))

  (define command/available? %command/available?)

  (extern-doc command/path
    (signature "(command/path name)")
    (summary "Resolve a command name to an executable path on the current host.")
    (param 'name "Command name to resolve.")
    (returns "The resolved executable path as a string, or `#f` when absent."))

  (define command/path %command/path)

  (doc-next
    (summary "Return whether a command resolves to an executable path.")
    (param 'name "Command name to resolve.")
    (returns "`#t` when the command resolves, otherwise `#f`."))

  (define (command/path? name) (if (%command/path name) #t #f))

  (doc-next
    (summary "Return whether a platform target matches the current host.")
    (param 'target "An OS symbol, a platform string, or a predicate object.")
    (returns "`#t` when the target matches the detected host, otherwise `#f`."))

  (define (host/matches? target)
    (cond
      ((symbol? target) (eq? target host/os))
      ((string? target) (string=? target host/platform))
      ((and (pair? target) (object/has-field? target 'os))
        (and
          (eq? (object/ref target 'os) host/os)
          (let ((target-arch (object/ref target 'arch #f)))
            (if target-arch (eq? target-arch host/arch) #t))))
      (else #f)))

  (extern-doc env/var
    (signature "(env/var name)")
    (summary "Return an environment variable from the current Scaffold process.")
    (param 'name "Environment variable name.")
    (returns "The variable value as a string, or `#f` when it is absent."))

  (define env/var %env/var)

  (doc-next
    (summary
      "Return whether an environment variable is set for the current Scaffold process.")
    (param 'name "Environment variable name.")
    (returns "`#t` when the variable is set, otherwise `#f`."))

  (define (env/var? name) (if (%env/var name) #t #f))

  (moduledoc
    (summary
      "Host facts and command availability helpers injected by the Rust runtime.")
    (group "Host")
    (effect 'host-read-only)
    (requires-capability 'scaffold.host)))
