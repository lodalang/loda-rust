; 1,3,6,10,15,21,21,29,38,48,59,71,84,84,99,115,132,150,169,189,189,211,234,258,283,309,336,336,365,395,426,458,491,525,525,561,598,636,675,715

add $0,1
mov $1,1
mov $3,$0
lpb $3
  mov $2,$0
  lpb $2
    sub $0,1
    dif $2,7
  lpe
  add $1,$0
  mov $3,$0
  sub $0,1
  sub $3,2
lpe
mov $0,$1
