; 1,1,2,3,4,5,6,7,7,8,8,9,10,10,11,12,13,13,14,15,16,17,17,18,19,20,21,22,22,23,24,25,26,27,28,28,29,30,31,32

mov $1,$0
add $0,1
mov $2,1
mov $3,$1
lpb $1
  sub $0,1
  mov $1,$3
  sub $1,6
  trn $1,1
  add $4,$2
  trn $3,1
  sub $3,$4
lpe
