(library
  (scaffold extensions support download)
  (export
    archive-extract-dir
    downloaded-archive-path
    download-bin/platform
    generated-shell-platform
    remote-bash-installer/platform
    sh-set
    sh/quote
    tool-cache-dir)
  (import (rnrs) (scaffold catalog base))

  (doc-next (summary "Return the per-tool Scaffold install cache directory."))

  (define (tool-cache-dir tool-name)
    (string-append "{{ state_dir }}/tools/" tool-name "/latest"))

  (doc-next (summary "Return the cached path for a downloaded archive."))

  (define (downloaded-archive-path tool-name archive-name)
    (string-append (tool-cache-dir tool-name) "/" archive-name))

  (doc-next (summary "Return the temporary archive extraction directory."))

  (define (archive-extract-dir tool-name)
    (string-append (tool-cache-dir tool-name) "/extract"))

  (doc-next
    (hidden)
    (summary "Return the POSIX shell-quoted representation of one character."))

  (define (sh/quoted-character character)
    (if (char=? character #\') "'\\''" (string character)))

  (doc-next
    (signature "(sh/quote value)")
    (summary "Single-quote a string for POSIX shell code."))

  (define (sh/quote value)
    (fold-right
      string-append
      "'"
      (cons "'" (map sh/quoted-character (string->list value)))))

  (doc-next
    (signature "(sh-set name value)")
    (summary "Return a POSIX shell assignment with a single-quoted value."))

  (define (sh-set name value) (string-append name "=" (sh/quote value) "\n"))

  (doc-next
    (summary "Create a package platform from a generated POSIX shell body.")
    (param 'predicate-value "Host predicate for this package rule.")
    (param 'requires "Commands required before this platform rule can run.")
    (param 'body "Shell script body appended after `set -eu`."))

  (define (generated-shell-platform predicate-value requires body)
    (package/platform
      predicate-value
      requires
      (arr "sh" "-c" (string-append "set -eu\n" body))))

  (doc-next
    (summary
      "Create a platform that downloads and executes an upstream Bash installer.")
    (param 'predicate-value "Host predicate for this package rule.")
    (param 'tool-name "Tool cache directory name.")
    (param 'url "Installer script URL.")
    (param 'env-vars "Vector of `NAME=value` entries passed through `env`.")
    (param 'args "Vector of arguments passed to the downloaded installer.")
    (param 'extra-dir "Optional directories to create before running the installer."))

  (define
    (remote-bash-installer/platform
      predicate-value
      tool-name
      url
      env-vars
      args
      .
      extra-dirs)
    (let
      ((root (tool-cache-dir tool-name))
        (script-path (string-append (tool-cache-dir tool-name) "/install.sh")))
      (package/platform-argvs
        predicate-value
        (arr "bash" "curl" "env" "mkdir")
        (arr
          (arr/append-list (arr "mkdir" "-p" root "{{ bin_dir }}") extra-dirs)
          (arr "curl" "-fsSL" "--retry" "3" "-o" script-path url)
          (vector/append (arr "env") env-vars (arr "bash" script-path) args)))))

  (doc-next
    (summary
      "Create a platform that downloads a file directly into `bin_dir` and chmods it executable.")
    (param 'predicate-value "Host predicate for this package rule.")
    (param 'url "Source file URL.")
    (param 'bin-name "Installed executable name."))

  (define (download-bin/platform predicate-value url bin-name)
    (package/platform-argvs
      predicate-value
      (arr "chmod" "curl" "mkdir")
      (arr
        (arr "mkdir" "-p" "{{ bin_dir }}")
        (arr
          "curl"
          "-fsSL"
          "--retry"
          "3"
          "-o"
          (string-append "{{ bin_dir }}/" bin-name)
          url)
        (arr "chmod" "+x" (string-append "{{ bin_dir }}/" bin-name)))))

  (moduledoc
    (summary "Download, cache, and generated shell package platform helpers.")
    (group "Download helpers")))
