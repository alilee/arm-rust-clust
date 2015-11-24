
first stack frame

    call seq                stack state                                         frame state:

    first_stack(ptos, fterm, ff):
    sub fp, fp, fp          nil_frame [<- fp]                                   nil_frame
    mov sp, a1              sp points past end of stack                         nil_frame
    mov lr, a2
    b a3

standard stack frame

                            p1, ..., pn [<- sp]; ? [<- fp]                      same as pre-b                      
    push fp                 p1..n, sfp [<- sp]; ? [<- fp]                       added fp 
    mov fp, sp              p1..n, sfp [<- sp, fp]                              new empty frame, fp points to previous frame
    push 0                  p1..n, sfp [<- fp], l1 [<- sp]                      
    push {r8,r9,r10, lr}    p1..n, sfp [<- fp], l1, r8, r9, r10, lr [<- sp]


    pop {r8,r9,r10,lr}
    add sp, sp, #4
    pop fp
    bx lr


garbage collection exception handler:

                            fp points to previous fp, or nil
                            sp points to current bottom of stack
                            registers may contain refs
                            
                            push a_regs, v_regs
                            current_frame = fp
                            a = sp
                            while current_frame > 0
                                 do 
                                     mark *a
                                     a += 1
                                 until a == current_frame
                                 a += 1
                                 current_frame = *current_frame
                                 
                                 
                                 
                                 
                            mark:


Extended frame

    ; Prologue - setup
    mov     ip, sp                 ; get a copy of sp.
    stm     sp!, {fp, ip, lr, pc}  ; Save the frame on the stack. See Addendum
    sub     fp, ip, #4             ; Set the new frame pointer.
        ...
    ; Maybe other functions called here.

    ; Older caller return lr stored in stack frame.
    bl      baz
        ...
    ; Epilogue - return
    ldm     sp, {fp, sp, lr}       ; restore stack, frame pointer and old link.
        ...                        ; maybe more stuff here.
    bx      lr                     ; return.
    

what about this?

    
    mov     ip, sp
    stm     sp!, {fp, ip, lr}
    sub     fp, ip, #4
    
    
    ldm     fp, {fp, sp, pc}
    