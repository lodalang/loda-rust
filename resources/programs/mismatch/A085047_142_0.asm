; 1,7,4,24,9,51,16,88,25,135,36,192,49,259,64,336,81,423,100,520,121,627,144,744,169,871,196,1008,225,1155,256,1312,289,1479,324,1656,361,1843,400,2040

mov $3,2
mov $7,$0
lpb $3
  mov $0,$7
  sub $3,1
  add $0,$3
  sub $0,1
  mov $4,3
  add $4,$0
  mov $5,$0
  add $0,1
  mov $2,$3
  div $4,2
  pow $4,2
  add $5,$0
  add $5,1
  mul $5,$4
  mov $6,$5
  lpb $2
    mov $1,$6
    sub $2,1
  lpe
lpe
lpb $7
  sub $1,$6
  mov $7,0
lpe
sub $1,2
div $1,2
add $1,1
mov $0,$1