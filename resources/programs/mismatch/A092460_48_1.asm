; 0,3,4,6,7,8,9,10,11,12,13,14,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43

mov $2,$0
lpb $0
  sub $2,1
  mov $4,$2
  mov $2,3
  lpb $4
    add $2,1
    mul $3,8
    add $3,1
    trn $4,$3
  lpe
  trn $2,4
  add $2,3
  add $2,$0
  mov $0,0
  sub $2,1
lpe
add $0,$1
add $0,3
mov $0,$2
