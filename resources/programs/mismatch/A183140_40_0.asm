; 0,0,0,1,2,3,5,7,9,11,14,17,20,24,28,32,36,41,46,51,57,63,69,76,83,90,97,105,113,121,130,139,148,157,167,177,187,198,209,220

mov $4,1
add $4,1
add $0,1
mov $5,1
mov $3,$0
lpb $3
  mov $4,24
  sub $5,1
  add $5,$4
  mov $1,$4
  mov $1,7
  mul $1,$3
  div $1,$5
  add $2,$1
  sub $3,1
  add $4,2
  mov $5,1
lpe
mov $0,$2
