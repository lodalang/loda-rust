; 20,40,41,60,61,62,80,81,82,83,100,101,102,103,104,120,121,122,123,124,125,140,141,142,143,144,145,146,160,161,162,163,164,165,166,167,180,181,182,183

mov $2,2
lpb $0
  add $1,1
  sub $0,$1
  add $2,2
lpe
mov $1,10
mul $1,$2
add $1,$0
mov $0,$1