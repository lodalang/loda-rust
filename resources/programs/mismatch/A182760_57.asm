; 1,3,5,6,8,10,12,13,15,17,18,20,22,24,25,27,29,31,32,34,36,37,39,41,43,44,46,48,49,51,53,55,56,58,60,62,63,65,67,68

mov $7,$0
add $0,1
mov $1,3
mov $3,2
mov $5,64
mov $6,10
lpb $0
  sub $0,1
  add $1,$3
  sub $1,$5
  add $1,4
lpe
add $1,1
add $2,5
mul $2,2
mov $4,1
sub $4,$6
mul $4,$6
add $4,$2
div $1,$4
add $1,1
add $1,$7
mov $0,$1