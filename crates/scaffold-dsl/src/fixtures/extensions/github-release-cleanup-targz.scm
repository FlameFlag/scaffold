(import (rnrs) (scaffold catalog) (scaffold extensions source github))
(object/ref
  (github/latest-targz-bin-platform
    'linux
    "demo"
    "owner/repo"
    "demo-${version}.tar.gz"
    "demo-${version}/demo"
    "demo")
  'install-argv)
