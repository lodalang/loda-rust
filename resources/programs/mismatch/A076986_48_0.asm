; 2,3,7,5,6,7,15,17,10,11,23,13,14,15,31,17,35,19,39,21,22,23,47,73,26,53,55,29,30,31,94,33,34,35,71,37,38,39,79,41

mov $2,$0
add $2,2
pow $2,2
mov $5,$0
mov $1,$0
sub $1,$0
lpb $2
  add $1,$5
  add $1,1
  mov $3,$1
  seq $3,7913
  sub $0,$3
  mov $4,$0
  max $4,0
  cmp $4,$0
  mul $2,$4
  sub $2,1
  max $2,3
lpe
mov $0,$1
add $0,1
