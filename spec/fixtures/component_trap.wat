(component
  (core module $m
    (func (export "unreachable") (unreachable))
  )
  (core instance $i (instantiate $m))
  (func $unreachable (canon lift (core func $i "unreachable")))
  (export "unreachable" (func $unreachable))
)
