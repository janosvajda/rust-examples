; Mini's tiny runtime, written directly in LLVM IR.
;
; Decimals are 64-bit whole numbers counting millionths: 2.5 is stored as 2500000.
; Adding, subtracting and comparing them is ordinary integer work, so codegen
; does that inline. Multiplying, dividing and printing need a little more, so
; they live here. Codegen appends this file to the program only when it uses
; decimals.

@.fmt_dec = private unnamed_addr constant [15 x i8] c"%s%lld.%0*lld\0A\00"
@.minus = private unnamed_addr constant [2 x i8] c"-\00"
@.no_sign = private unnamed_addr constant [1 x i8] c"\00"

; a * b: the product of two millionths counts millionths of millionths, so divide
; by 1000000 once. The product can be larger than 64 bits, so work in 128 bits.
define private i64 @mini_dec_mul(i64 %a, i64 %b) {
entry:
  %a128 = sext i64 %a to i128
  %b128 = sext i64 %b to i128
  %product = mul i128 %a128, %b128
  %scaled = sdiv i128 %product, 1000000
  %result = trunc i128 %scaled to i64
  ret i64 %result
}

; a / b: multiply a by 1000000 first, so the quotient keeps its six decimal places.
; Digits after the sixth place are cut off, like whole-number division.
define private i64 @mini_dec_div(i64 %a, i64 %b) {
entry:
  %zero = icmp eq i64 %b, 0
  br i1 %zero, label %trap, label %divide
trap:
  call void @llvm.trap()
  unreachable
divide:
  %a128 = sext i64 %a to i128
  %b128 = sext i64 %b to i128
  %a_scaled = mul i128 %a128, 1000000
  %quotient = sdiv i128 %a_scaled, %b128
  %result = trunc i128 %quotient to i64
  ret i64 %result
}

; Print a decimal without trailing zeros: 2500000 prints as "2.5", 2000000 as "2.0".
define private void @mini_print_dec(i64 %value) {
entry:
  ; split into sign, whole part and fraction: -2.5 → "-", 2, 500000
  %negative = icmp slt i64 %value, 0
  %negated = sub i64 0, %value
  %abs = select i1 %negative, i64 %negated, i64 %value
  %sign = select i1 %negative, ptr @.minus, ptr @.no_sign
  %whole = udiv i64 %abs, 1000000
  %fraction = urem i64 %abs, 1000000
  br label %strip

strip:
  ; drop trailing zeros from the fraction, keeping at least one digit
  %frac = phi i64 [ %fraction, %entry ], [ %frac_shorter, %drop_zero ]
  %digits = phi i32 [ 6, %entry ], [ %digits_shorter, %drop_zero ]
  %last_digit = urem i64 %frac, 10
  %ends_in_zero = icmp eq i64 %last_digit, 0
  %more_than_one = icmp ugt i32 %digits, 1
  %can_drop = and i1 %ends_in_zero, %more_than_one
  br i1 %can_drop, label %drop_zero, label %print

drop_zero:
  %frac_shorter = udiv i64 %frac, 10
  %digits_shorter = sub i32 %digits, 1
  br label %strip

print:
  ; printf("%s%lld.%0*lld\n", sign, whole, digits, frac): %0*lld pads with zeros, so 0.05 keeps its 0
  call i32 (ptr, ...) @printf(ptr @.fmt_dec, ptr %sign, i64 %whole, i32 %digits, i64 %frac)
  ret void
}
