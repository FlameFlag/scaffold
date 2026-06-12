(library
  (software-packaging distrobox git)
  (export arch-custom-git fedora-custom-git debian-custom-git)
  (import
    (rnrs)
    (scaffold catalog)
    (software-packaging distrobox boxes)
    (software-packaging support shell))

  (doc-next
    (hidden)
    (summary "Build a Distrobox-wrapped Git source build example tool."))

  (define (git-in-box name box box-tool native-packages-tool)
    (in-box
      box
      box-tool
      (tool
        name
        (package
          (field 'name name)
          (field
            'install-argvs
            (arr
              (shell-command
                "cd /workdir/sources/git && make configure && ./configure --prefix=$HOME/.local/scaffold-example/git --without-tcltk")
              (shell-command
                "cd /workdir/sources/git && make -j${JOBS:-2} NO_GETTEXT=YesPlease NO_TCLTK=YesPlease all")
              (shell-command
                "cd /workdir/sources/git && make NO_GETTEXT=YesPlease NO_TCLTK=YesPlease install"))))
        (field 'bins (arr (bin "git")))
        (field 'checks (arr (check (arr "git" "--version")))))
      native-packages-tool))

  (doc-next (summary "Example Arch Distrobox Git tool."))

  (define arch-custom-git
    (git-in-box
      "arch-custom-git"
      "scaffold-arch"
      "scaffold-arch"
      "arch-native-packages"))

  (doc-next (summary "Example Fedora Distrobox Git tool."))

  (define fedora-custom-git
    (git-in-box
      "fedora-custom-git"
      "scaffold-fedora"
      "scaffold-fedora"
      "fedora-native-packages"))

  (doc-next (summary "Example Debian Distrobox Git tool."))

  (define debian-custom-git
    (git-in-box
      "debian-custom-git"
      "scaffold-debian"
      "scaffold-debian"
      "debian-native-packages")))
