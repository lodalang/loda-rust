; 1,3,6,10,15,21,28,36,45,55,66,78,91,105,120,137,155,174,194,215,237,260,284,309,335,362,390,419,449,480,513,547,582,618,655,693,732,772,813,855

mov $1,1
lpb $0
  mov $2,$0
  sub $0,1
  lpb $2
    add $1,$2
    div $2,15
  lpe
  add $1,1
lpe
mov $0,$1
