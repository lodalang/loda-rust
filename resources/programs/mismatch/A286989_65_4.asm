; 1,2,4,5,7,9,10,12,14,15,17,18,20,22,23,25,27,28,30,31,33,35,36,38,40,41,43,45,46,48,49,51,53,54,56,58,59,61,62,64

mov $2,7
mov $3,$0
mov $5,1
add $5,$0
lpb $2
  mov $6,$5
  lpb $5
    mov $5,$4
    pow $6,2
  lpe
  mov $0,5
  mov $1,5
  mov $5,1
  lpb $6
    add $0,1
    add $5,$1
    trn $6,$5
  lpe
  mov $2,1
lpe
sub $0,3
add $0,$3
sub $0,2