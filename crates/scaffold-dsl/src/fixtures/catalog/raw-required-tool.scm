(import (rnrs) (scaffold config))

(moduledoc (summary "Fixture for a raw catalog object with a required action."))

(let ((field cons) (object list) (arr vector))
  (object
    (field
      'tools
      (arr
        (object
          (field 'name "demo")
          (field
            'bins
            (arr
              (object
                (field 'name "demo")
                (field 'version-argv (arr "demo" "--version")))))
          (field 'action (object (field 'type 'required))))))))
