(library
  (scaffold extensions platform windows registry)
  (export reg/tool reg/query-argv reg/add-argv)
  (import (rnrs) (scaffold catalog base) (scaffold extensions platform windows base))

  (doc-next
    (signature "(reg/tool field ...)")
    (summary "Create a required descriptor for Windows Registry CLI `reg`."))

  (define reg/tool (windows/command-tool-proc "reg"))

  (doc-next
    (summary "Build argv for querying one Windows Registry value.")
    (param 'key "Registry key path.")
    (param 'value-name "Registry value name.")
    (returns "Vector argv for `reg query`."))

  (define (reg/query-argv key value-name) (arr "reg" "query" key "/v" value-name))

  (doc-next
    (summary "Build argv for adding or replacing one Windows Registry value.")
    (param 'key "Registry key path.")
    (param 'value-name "Registry value name.")
    (param 'type "Registry data type such as `REG_SZ` or `REG_DWORD`.")
    (param 'data "Value data.")
    (returns "Vector argv for `reg add /f`."))

  (define (reg/add-argv key value-name type data)
    (arr "reg" "add" key "/v" value-name "/t" type "/d" data "/f"))

  (moduledoc
    (summary "Windows Registry command descriptors and argv helpers.")
    (group "Windows tools")))
