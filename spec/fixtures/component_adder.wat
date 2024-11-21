(component
  ;; Define a nested component so we can export an instance of a component
  (component $c
    (core module $m
        (func (export "add") (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.add
        )
    )
    (core instance $i (instantiate $m))
    (func $add (param "a" s32) (param "b" s32) (result s32) (canon lift (core func $i "add")))
    (export "add" (func $add))
  )
  (instance $adder (instantiate $c))

  ;; Export the adder instance
  (export "adder" (instance $adder))

  ;; Re-export add as a top level
  (export "add" (func $adder "add"))
)
