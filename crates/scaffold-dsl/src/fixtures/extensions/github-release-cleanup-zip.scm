(import (rnrs) (scaffold catalog) (scaffold extensions source github))
(object/ref
  (github/latest-zip-bin-platform
    'linux
    "demo"
    "owner/repo"
    "demo-${version}.zip"
    "demo-${version}/demo"
    "demo")
  'install-argv)
