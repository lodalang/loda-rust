; 1,2,3,3,5,6,7,4,7,10,11,9,13,14,15,5,17,14,19,15,21,22,23,12,21,26,15,21,29,30,31,6,33,34,35,21,37,38,39,20

mov $1,1
mov $2,2
mov $4,1
add $0,1
lpb $0
  mov $3,$0
  lpb $3
    mov $4,$0
    mod $4,$2
    cmp $4,0
    cmp $4,0
    add $2,1
    sub $3,$4
    cmp $6,3
  lpe
  mov $5,1
  lpb $0
    dif $0,$2
    sub $6,1
    add $4,$2
    div $4,$2
    div $4,2
    sub $4,$5
    add $4,1
    mul $5,$2
    add $5,$4
  lpe
  mul $1,$5
lpe
mov $0,$1
