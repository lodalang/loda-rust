; 1,3,4,6,7,8,10,11,13,14,15,17,18,19,21,22,23,25,26,27,28,30,31,32,34,35,36,37,39,40,41,42,44,45,46,47,49,50,51,52

mov $1,$0
mul $1,5
sub $1,2
mov $2,10
lpb $1
  add $0,1
  add $2,1
  trn $1,$2
lpe
add $0,1
