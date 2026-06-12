(library
  (software-packaging source git)
  (export custom-git-linux custom-git-macos mingw-w64-toolchain custom-git-windows)
  (import
    (rnrs)
    (scaffold catalog)
    (scaffold extensions app winget)
    (software-packaging support shell))

  (define git-source-path "sources/git")

  (define msys2-bash "C:/msys64/usr/bin/bash.exe")

  (doc-next (hidden) (summary "Build an MSYS2 Bash argv vector for a shell command."))

  (define (msys2-command command) (arr msys2-bash "-lc" command))

  (doc-next
    (hidden)
    (summary "Build an MSYS2 MinGW64 shell argv vector for a command."))

  (define (mingw64-command command)
    (msys2-command
      (string-append
        "export MSYSTEM=MINGW64; export PATH=/mingw64/bin:/usr/bin:$PATH; "
        command)))

  (doc-next
    (hidden)
    (summary "Create a Git source build example tool for one platform."))

  (define (git-build-tool name platform configure-command build-command install-command)
    (tool
      name
      (build
        (field 'path git-source-path)
        (field
          'argvs
          (arr
            (shell-command configure-command)
            (shell-command build-command)
            (shell-command install-command))))
      (tool/platforms platform)
      (field 'bins (arr (bin "git")))
      (field 'verify-after-install #f)
      (meta
        (description "Minimal Git source build recipe.")
        (home-page "https://git-scm.com/")
        (source "https://mirrors.edge.kernel.org/pub/software/scm/git/")
        (tags "source" "git"))))

  (doc-next (summary "Example Linux Git build tool."))

  (define custom-git-linux
    (git-build-tool
      "custom-git-linux"
      'linux
      "make configure && ./configure --prefix={{ prefix }} --without-tcltk"
      "make -j${JOBS:-2} NO_GETTEXT=YesPlease NO_TCLTK=YesPlease all"
      "make NO_GETTEXT=YesPlease NO_TCLTK=YesPlease install"))

  (doc-next (summary "Example macOS Git build tool."))

  (define custom-git-macos
    (git-build-tool
      "custom-git-macos"
      'macos
      "make configure && ./configure --prefix={{ prefix }} --without-tcltk"
      "make -j${JOBS:-2} NO_GETTEXT=YesPlease NO_TCLTK=YesPlease all"
      "make NO_GETTEXT=YesPlease NO_TCLTK=YesPlease install"))

  (doc-next (summary "Example Windows Git package tool."))

  (define mingw-w64-toolchain
    (tool
      "mingw-w64-toolchain"
      (package
        (field 'name "MSYS2.MSYS2")
        (field
          'install-argvs
          (arr
            (winget/install-argv "{{ package }}")
            (msys2-command "pacman --noconfirm -Syuu")
            (msys2-command
              "pacman --noconfirm -S --needed base-devel perl mingw-w64-x86_64-toolchain mingw-w64-x86_64-curl mingw-w64-x86_64-expat mingw-w64-x86_64-openssl mingw-w64-x86_64-pcre2 mingw-w64-x86_64-zlib"))))
      (tool/platforms 'windows)
      (field 'bins (arr (bin "gcc") (bin "make")))
      (field 'checks (arr (host/check 'windows (mingw64-command "gcc --version"))))
      (meta
        (description
          "MSYS2 MinGW-w64 compiler toolchain used for Windows source builds.")
        (home-page "https://www.msys2.org/")
        (source "https://packages.msys2.org/")
        (tags "source" "windows" "mingw" "compiler"))))

  (doc-next (summary "Example Windows Git source build tool."))

  (define custom-git-windows
    (tool
      "custom-git-windows"
      (build
        (field 'path git-source-path)
        (field
          'argvs
          (arr
            (mingw64-command
              "make configure && ./configure --prefix='{{ prefix }}' --without-tcltk")
            (mingw64-command
              "make -j${JOBS:-2} NO_GETTEXT=YesPlease NO_TCLTK=YesPlease all")
            (mingw64-command "make NO_GETTEXT=YesPlease NO_TCLTK=YesPlease install"))))
      (tool/platforms 'windows)
      (depends "mingw-w64-toolchain")
      (field 'bins (arr (bin "git")))
      (field 'verify-after-install #f)
      (meta
        (description "Minimal Git source build recipe for MinGW-w64 on Windows.")
        (home-page "https://git-scm.com/")
        (source "https://mirrors.edge.kernel.org/pub/software/scm/git/")
        (tags "source" "git" "windows" "mingw")))))
