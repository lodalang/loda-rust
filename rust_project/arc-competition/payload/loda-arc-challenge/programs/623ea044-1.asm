; Submitted by Simon Strandgaard
; Program Type: simple

mov $20,255 ; color when there is no neighbour

; ignore mask
mov $1,$0
mov $2,$0
f11 $2,101060 ; most popular color
f21 $1,101250 ; mask where color is
; $2 is most popular color
; $1 is the ignore mask

; neighbour_up_left
mov $10,$0
mov $11,$1
mov $12,$20
f31 $10,102064 ; neighbour 'UpLeft'
mov $3,$10

; neighbour_up_right
mov $10,$0
mov $11,$1
mov $13,$20
f31 $10,102065 ; neighbour 'UpRight'
mov $4,$10

; neighbour_down_left
mov $10,$0
mov $11,$1
mov $13,$20
f31 $10,102066 ; neighbour 'DownLeft'
mov $5,$10

; neighbour_down_right
mov $10,$0
mov $11,$1
mov $13,$20
f31 $10,102067 ; neighbour 'DownRight'
mov $6,$10

; prepare the output image
mov $14,$0 ; clone input image

; draw diagonal line - down right
mov $15,$3 ; neighbour_up_left
mov $16,$20
f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter. 

; draw diagonal line - down left
mov $15,$4 ; neighbour_up_right
mov $16,$20
f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter. 

; draw diagonal line - up right
mov $15,$5 ; neighbour_down_left
mov $16,$20
f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter. 

; draw diagonal line - up left
mov $15,$6 ; neighbour_down_right
mov $16,$20
f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter. 

mov $0,$14
