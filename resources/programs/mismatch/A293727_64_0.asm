; 1,3,4,5,6,7,8,9,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64

add $1,5
mov $1,$0
add $2,16
mov $3,1
lpb $0
  div $0,8
  mov $2,$0
  mov $2,1
  add $2,1
  mul $2,$3
  sub $2,1
  add $1,$2
  mul $3,12
lpe
add $1,1
mov $0,$1
