; 2,5,7,10,12,15,18,20,23,25,28,30,33,36,38,41,43,46,49,51,54,56,59,61,64,67,69,72,74,77,80,82,85,87,90,92,95,98,100,103

mov $7,$0
add $0,1
mov $2,2
mov $3,3
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
sub $4,6
mul $4,$6
add $4,$2
div $1,$4
add $1,1
add $1,$7
mov $0,$1