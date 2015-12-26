
clojure like, but without java interop
pixie

completely immutable?
compiler?
garbage collecting leveraging OS garbage collector?

    > 2
    2

    > (+ 2 2)
    4

    > (def x 3)
    'x

    > (+ 2 x)
    5

    > ()
    ()

    > (cons 2 ())
    (2)

    > (cons 2 (3))
    (2 3)

    > (rest (cons 2 (3)))
    (3)
    
    > [2 3]
    [2 3]

    > (let [x 3] (+ 2 x))
    5