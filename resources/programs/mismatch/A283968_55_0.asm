; 1,2,3,5,7,9,12,15,19,23,27,32,37,42,48,54,61,68,75,83,91,100,109,118,128,138,148,159,170,182,194,206,219,232,245,259,273,288,303,318

mov $2,$0
mov $3,$0
add $3,1
lpb $3
  sub $3,1
  mov $0,$2
  sub $0,$3
  mov $4,$0
  mul $0,8
  mul $4,112
  div $4,6
  add $0,$4
  div $4,49
  add $4,1
  add $1,$4
lpe
mov $0,$1
