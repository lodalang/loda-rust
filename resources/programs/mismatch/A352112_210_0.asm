; 0,1,-2,-1,-4,-3,6,7,4,5,2,3,12,13,10,11,8,9,18,19,16,17,14,15,24,25,22,23,20,21,-30,-29,-32,-31,-34,-33,-24,-23,-26,-25

mov $1,$0
mov $2,2
mov $3,4
lpb $0
  div $0,$2
  mul $2,2
  div $3,-1
  mov $4,$0
  mul $4,$3
  sub $2,1
  add $1,$4
  mul $3,$2
lpe
mov $0,$1
