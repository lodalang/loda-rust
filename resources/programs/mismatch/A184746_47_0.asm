; 1,3,4,6,7,9,10,12,13,14,16,17,19,20,22,23,25,26,27,29,30,32,33,35,36,38,39,41,42,43,45,46,48,49,51,52,54,55,56,58

mov $1,3
mov $2,$0
add $2,6
mov $3,$0
add $0,$2
add $2,4
add $1,$2
mul $1,7
mov $4,20
lpb $0
  add $1,$0
  trn $0,$1
  add $5,$4
  sub $0,0
  div $1,$5
lpe
sub $1,3
add $1,$3
mov $0,1
mov $0,$1