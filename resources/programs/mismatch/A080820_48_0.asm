; 1,2,3,4,4,5,6,6,7,8,9,9,10,11,11,12,13,14,14,15,16,16,17,18,19,19,20,21,21,22,23,23,24,25,26,26,27,28,28,29

mov $3,$0
mul $0,2
add $0,4
add $0,$3
mov $1,$0
lpb $1
  sub $1,14
  add $2,4
  sub $0,1
  sub $1,$2
  trn $1,1
  mov $2,0
lpe
div $0,4
add $0,2
sub $0,1
