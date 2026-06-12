(import
  (rnrs)
  (scaffold catalog)
  (scaffold test)
  (software-packaging desktop vscode)
  (software-packaging distrobox git)
  (software-packaging distrobox native)
  (software-packaging distrobox nix)
  (software-packaging distrobox vscode)
  (software-packaging ecosystems tools)
  (software-packaging source git))

(doc-next (summary "Return the action object from a tool value."))

(define (action tool-value) (object/ref tool-value 'action))

(doc-next (summary "Return the install argv from a tool action."))

(define (install-argv tool-value) (object/ref (action tool-value) 'install-argv))

(doc-next (summary "Return ordered install argv vectors from a tool action."))

(define (install-argvs tool-value) (object/ref (action tool-value) 'install-argvs))

(doc-next (summary "Return ordered build argv vectors from a build action."))

(define (build-argvs tool-value) (object/ref (action tool-value) 'argvs))

(doc-next (summary "Return a check argv by index from a tool object."))

(define (check-argv tool-value index)
  (object/ref (vector-ref (object/ref tool-value 'checks) index) 'argv))

(assert/equal
  (arr "sh" "-lc" "make configure && ./configure --prefix={{ prefix }} --without-tcltk")
  (vector-ref (build-argvs custom-git-linux) 0))

(assert/equal
  (arr
    "winget"
    "install"
    "--id"
    "{{ package }}"
    "--exact"
    "--silent"
    "--accept-package-agreements"
    "--accept-source-agreements")
  (vector-ref (install-argvs mingw-w64-toolchain) 0))

(assert/equal
  (arr
    "C:/msys64/usr/bin/bash.exe"
    "-lc"
    "pacman --noconfirm -S --needed base-devel perl mingw-w64-x86_64-toolchain mingw-w64-x86_64-curl mingw-w64-x86_64-expat mingw-w64-x86_64-openssl mingw-w64-x86_64-pcre2 mingw-w64-x86_64-zlib")
  (vector-ref (install-argvs mingw-w64-toolchain) 2))

(assert/equal
  (arr
    "C:/msys64/usr/bin/bash.exe"
    "-lc"
    "export MSYSTEM=MINGW64; export PATH=/mingw64/bin:/usr/bin:$PATH; make configure && ./configure --prefix='{{ prefix }}' --without-tcltk")
  (vector-ref (build-argvs custom-git-windows) 0))

(assert/equal
  "mingw-w64-toolchain"
  (vector-ref (object/ref custom-git-windows 'depends) 0))

(assert/equal
  (arr "flatpak" "install" "--assumeyes" "--noninteractive" "flathub" "{{ package }}")
  (object/ref (vector-ref (object/ref (action vscode) 'platforms) 0) 'install-argv))

(assert/equal
  (arr
    "distrobox"
    "enter"
    "scaffold-arch"
    "--"
    "sudo"
    "pacman"
    "-S"
    "--needed"
    "--noconfirm"
    "base-devel"
    "curl"
    "jq"
    "nix"
    "code")
  (install-argv arch-native-packages))

(assert/equal
  (arr
    "distrobox"
    "enter"
    "scaffold-fedora"
    "--"
    "sudo"
    "dnf"
    "install"
    "-y"
    "gcc"
    "make"
    "curl"
    "jq"
    "nix"
    "code")
  (install-argv fedora-native-packages))

(assert/equal
  (arr "distrobox" "enter" "scaffold-debian" "--" "sudo" "apt-get" "update")
  (vector-ref (install-argvs debian-native-packages) 0))

(assert/equal
  (arr
    "distrobox"
    "enter"
    "scaffold-arch"
    "--"
    "nix"
    "--extra-experimental-features"
    "nix-command flakes"
    "profile"
    "add"
    "nixpkgs#ripgrep"
    "nixpkgs#fd"
    "nixpkgs#jq")
  (install-argv arch-nix-profile-basics))

(assert/equal
  (arr
    "distrobox"
    "enter"
    "scaffold-arch"
    "--"
    "sh"
    "-lc"
    "cd /workdir/sources/git && make configure && ./configure --prefix=$HOME/.local/scaffold-example/git --without-tcltk")
  (vector-ref (install-argvs arch-custom-git) 0))

(assert/equal
  (arr "distrobox" "enter" "scaffold-arch" "--" "git" "--version")
  (check-argv arch-custom-git 0))

(assert/equal
  (arr
    "distrobox"
    "enter"
    "scaffold-debian"
    "--"
    "sudo"
    "apt-get"
    "install"
    "-y"
    "{{ package }}")
  (install-argv debian-vscode))

(assert/equal (arr "bun" "add" "-g" "{{ package }}") (install-argv bun-typescript))

(assert/equal
  (arr "uv" "tool" "install" "--upgrade" "--force" "{{ package }}")
  (install-argv uv-ruff))
