; 2,4,8,10,14,16,20,22,26,28,30,32,34,38,40,44,46,50,52,56,58,60,62,64,68,70,74,76,80,82,86,88,90,92,94,98,100,104,106,110

add $0,1
mov $1,1
mov $4,$0
mov $5,$0
lpb $0
  mov $0,4
  mul $0,$4
  mov $2,$4
  mod $2,11
  add $0,$2
  div $0,11
  mov $1,$0
lpe
mov $3,$5
mul $3,2
add $1,$3
add $0,$1
