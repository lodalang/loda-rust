; 0,1,2,3,4,6,8,9,10,11,12,13,14,15,17,19,20,21,22,23,24,25,26,28,30,31,32,33,34,35,36,37,39,41,42,43,44,45,46,47

mov $2,$0
add $0,2
mov $1,$2
mov $3,$4
lpb $0
  sub $0,6
  trn $0,$3
  add $1,$0
  trn $0,1
  add $1,1
  sub $1,$0
  mov $3,2
lpe
lpb $2
  add $1,4
  sub $3,1
lpe
sub $1,1
mov $0,$1