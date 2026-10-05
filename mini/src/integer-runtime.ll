; Mini's guarded whole-number division, written directly in LLVM IR.
;
; LLVM's `sdiv` has undefined behaviour in two cases: dividing by zero, and
; -2147483648 / -1, whose answer (2147483648) doesn't fit in an i32. Mini
; doesn't allow either: the program stops with a trap instead. Codegen appends
; this file to the program only when it divides whole numbers.

; a / b, rounded toward zero, like Rust's `/` on i32.
define private i32 @mini_int_div(i32 %a, i32 %b) {
entry:
  %zero = icmp eq i32 %b, 0
  %a_is_min = icmp eq i32 %a, -2147483648
  %b_is_minus_one = icmp eq i32 %b, -1
  %overflow = and i1 %a_is_min, %b_is_minus_one
  %invalid = or i1 %zero, %overflow
  br i1 %invalid, label %trap, label %divide
trap:
  call void @llvm.trap()
  unreachable
divide:
  %quotient = sdiv i32 %a, %b
  ret i32 %quotient
}
