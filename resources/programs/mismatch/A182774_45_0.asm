; 1,3,4,6,8,9,11,13,14,16,17,19,21,22,24,26,27,29,30,32,34,35,37,39,40,42,44,45,47,48,50,52,53,55,57,58,60,61,63,65

mov $7,$0
add $0,1
mov $2,3
mov $3,1
mov $5,64
mov $6,10
lpb $0
  sub $0,1
  add $1,$3
  sub $1,$5
  add $1,10
lpe
add $1,1
add $2,5
mul $2,2
sub $4,$6
mul $4,$6
add $4,$2
div $1,$4
add $1,1
add $1,$7
mov $0,$1