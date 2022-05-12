require 'net/http'
require 'uri'
require 'json'

BASE_URL = "http://api.loda-lang.org/miner/v1/"
uri = URI.parse(BASE_URL + "programs")

header = {
    'Content-Type': 'application/octet-stream'
}

content = <<CONTENT
; A047315: Numbers that are congruent to {2, 4, 5, 6} mod 7.
; Submitted by Simon Strandgaard
; 2,4,5,6,9,11,12,13,16,18,19,20,23,25,26,27,30,32,33,34,37,39,40,41,44,46,47,48,51,53,54,55,58,60,61,62,65,67,68,69,72,74,75,76,79,81,82,83,86,88,89,90,93,95,96,97,100,102,103,104,107,109,110,111

mul $0,5
div $0,4
mul $0,7
add $0,13
div $0,5
CONTENT

content.strip!

http = Net::HTTP.new(uri.host, uri.port)
request = Net::HTTP::Post.new(uri.request_uri, header)
request.body = content

start = Time.now
response = http.request(request)
elapsed = Time.now - start

p response
p elapsed

