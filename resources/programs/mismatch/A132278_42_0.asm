; 1,1,2,3,5,7,10,13,17,21,26,31,37,43,49,56,64,72,81,90,100,110,121,132,144,156,169,182,196,210,225,240,256,272,289,306,324,342,361,380

mul $1,1682
mov $1,1
sub $1,1
mov $2,$0
div $0,14
seq $0,78012
lpb $2
  sub $2,1
  add $0,$2
  sub $2,1
  mov $1,10
lpe
