; 0,1,3,6,10,15,21,28,29,31,34,38,43,49,56,64,66,69,73,78,84,91,99,108,111,115,120,126,133,141,150,160,164,169,175,182,190,199,209,220

mov $4,$0
pow $2,2
mov $3,$0
lpb $3
  sub $3,1
  mov $0,$4
  sub $0,$3
  mov $2,$0
  lpb $0
    mod $0,8
    div $2,8
    add $2,$0
  lpe
  add $1,$2
lpe
mov $0,$1
