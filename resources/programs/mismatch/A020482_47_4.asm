; 2,3,5,7,7,11,13,13,17,19,19,23,23,23,29,31,31,31,37,37,41,43,43,47,47,47,53,53,53,59,61,61,61,67,67,71,73,73,73,79

add $0,1
mov $1,$0
mov $2,$0
lpb $2
  mov $3,$2
  gcd $3,$1
  cmp $3,1
  add $1,$3
  sub $2,1
lpe
mov $0,$1